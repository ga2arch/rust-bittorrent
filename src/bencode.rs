use nom::{
    IResult,
    Err,
    bytes::complete::{tag, is_not, take},
    sequence::{tuple, delimited},
    multi::{many0},
    branch::alt,
    error::{ErrorKind, ParseError},
    Err::{Failure}};
use nom::multi::fold_many0;
use nom::lib::std::collections::{BTreeMap, HashMap};
use indexmap::map::IndexMap;

#[derive(Debug, PartialEq)]
pub enum BencodeParserError<'a> {
    InvalidPrefixNumber,
    InvalidByteString,
    Nom(&'a [u8], ErrorKind),
}

impl<'a> ParseError<&'a [u8]> for BencodeParserError<'a> {
    fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
        BencodeParserError::Nom(input, kind)
    }

    fn append(_: &'a [u8], _: ErrorKind, other: Self) -> Self {
        other
    }
}

type BencodeParserResult<'a> = IResult<&'a [u8], BencodeValue<'a>, BencodeParserError<'a>>;

trait BencodeParser<'a> {
    fn parse(&self) -> BencodeParserResult<'a>;
}

struct BencodeParserBytes<'a> {
    input: &'a [u8]
}

impl<'a> BencodeParser<'a> for BencodeParserBytes<'a> {
    fn parse(&self) -> BencodeParserResult<'a> {
        parse(self.input)
    }
}

pub fn from_bytes(input: &[u8]) -> BencodeParserResult {
    BencodeParserBytes { input }.parse()
}

#[derive(Debug, PartialEq, Clone)]
pub enum BencodeValue<'a> {
    Integer(i64),
    ByteString(&'a [u8]),
    List(Vec<BencodeValue<'a>>),
    Dict(IndexMap<&'a [u8], BencodeValue<'a>>),
}

fn bytes_to_i64(bytes: &[u8]) -> Result<i64, Err<BencodeParserError>> {
    std::str::from_utf8(bytes)
        .map_err(|_| Failure(BencodeParserError::InvalidPrefixNumber))?
        .parse::<i64>()
        .map_err(|_| Failure(BencodeParserError::InvalidPrefixNumber))
}

fn parse(input: &[u8]) -> BencodeParserResult {
    alt((parse_dict, parse_list, parse_byte_string, parse_integer))(input)
}

fn parse_integer(input: &[u8]) -> BencodeParserResult {
    let (input, parsed) = delimited(tag("i"), is_not("e"), tag("e"))(input)?;
    Ok((input, BencodeValue::Integer(bytes_to_i64(parsed)?)))
}

fn parse_byte_string(input: &[u8]) -> BencodeParserResult {
    let (input, (len, _)) = tuple((is_not(":ilde"), tag(":")))(input)?;
    let len = bytes_to_i64(len)?;
    if len >= 0 {
        let (input, byte_string) = take(len as u64)(input)?;
        Ok((input, BencodeValue::ByteString(byte_string)))
    } else {
        Err(Failure(BencodeParserError::InvalidByteString))?
    }
}

fn parse_list(input: &[u8]) -> BencodeParserResult {
    let (input, xs) =
        delimited(tag("l"), many0(parse), tag("e"))(input)?;
    Ok((input, BencodeValue::List(xs)))
}

fn parse_dict(input: &[u8]) -> BencodeParserResult {
    let kv_parser = fold_many0(tuple((parse_byte_string, parse)), IndexMap::new(),
                               |mut acc: IndexMap<&[u8], BencodeValue>, (key, value)| {
                                   match key {
                                       BencodeValue::ByteString(bs) => {
                                           acc.insert(bs, value.clone());
                                           acc
                                       }
                                       _ => acc
                                   }
                               });

    let (input, dict): (&[u8], IndexMap<&[u8], BencodeValue>) = delimited(tag("d"), kv_parser, tag("e"))(input)?;
    Ok((input, BencodeValue::Dict(dict)))
}

pub fn to_bytes(value: &BencodeValue) -> Vec<u8> {
    match value {
        BencodeValue::Integer(num) => {
            let mut out = Vec::new();
            out.push('i' as u8);
            out.extend_from_slice(num.to_string().as_bytes());
            out.push('e' as u8);
            out
        }
        BencodeValue::ByteString(bs) => {
            let mut out = Vec::new();
            out.extend_from_slice(bs.len().to_string().as_bytes());
            out.push(':' as u8);
            out.extend_from_slice(bs);
            out
        }
        BencodeValue::List(xs) => {
            let mut out = Vec::new();
            out.push('l' as u8);
            for x in xs.iter() {
                out.extend_from_slice(to_bytes(x).as_slice());
            }
            out.push('e' as u8);
            out
        }
        BencodeValue::Dict(kv) => {
            let mut out = Vec::new();
            out.push('d' as u8);
            for (k, v) in kv.iter() {
                out.extend_from_slice(to_bytes(&BencodeValue::ByteString(k)).as_slice());
                out.extend_from_slice(to_bytes(v).as_slice());
            }
            out.push('e' as u8);
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bencode::{BencodeValue, BencodeParserError, from_bytes, to_bytes, BencodeParserResult};
    use nom::Err::{Failure};
    use nom::sequence::delimited;
    use indexmap::map::IndexMap;
    use std::error::Error;

    #[test]
    fn parse_integer() {
        //given
        let input = "i42e".as_bytes();

        //when
        let result = from_bytes(input);

        //then
        assert_eq!(result, Ok(("".as_bytes(), BencodeValue::Integer(42i64))))
    }

    #[test]
    fn parse_string() {
        //given
        let input = "4:spam".as_bytes();

        //when
        let result = from_bytes(input);

        //then
        assert_eq!(result, Ok(("".as_bytes(), BencodeValue::ByteString("spam".as_bytes()))))
    }

    #[test]
    fn parse_negative_string() {
        //given
        let input = "-1:spam".as_bytes();

        //when
        let result = from_bytes(input);

        //then
        assert_eq!(result, Err(Failure(BencodeParserError::InvalidByteString)))
    }

    #[test]
    fn parse_list() {
        //given
        let input = "l4:spami42ee".as_bytes();

        //when
        let result = from_bytes(input);

        //then
        let elem1 = BencodeValue::ByteString("spam".as_bytes());
        let elem2 = BencodeValue::Integer(42i64);
        assert_eq!(result, Ok(("".as_bytes(), BencodeValue::List(vec![elem1, elem2]))))
    }

    #[test]
    fn parse_dict() {
        //given
        let input = "d3:bar4:spam3:fooi42ee".as_bytes();

        //when
        let result = from_bytes(input);

        //then
        let key1 = "bar".as_bytes();
        let val1 = BencodeValue::ByteString("spam".as_bytes());

        let key2 = "foo".as_bytes();
        let val2 = BencodeValue::Integer(42i64);

        let mut map = IndexMap::new();
        map.insert(key1, val1);
        map.insert(key2, val2);

        assert_eq!(result, Ok(("".as_bytes(), BencodeValue::Dict(map))))
    }

    #[test]
    fn parse_complex() {
        //given
        let input = include_bytes!("../resources/archlinux-2020.06.01-x86_64.iso.torrent");

        //when
        let result = from_bytes(input);

        //then
        match result {
            Ok(_) => debug_assert!(true, "correctly parsed"),
            Err(err) => debug_assert!(false, "error: {}", err)
        }
    }

    #[test]
    fn serialize_integer() {
        //given
        let input = BencodeValue::Integer(10);

        //when
        let result = to_bytes(&input);

        //then
        assert_eq!(result, "i10e".as_bytes().to_vec())
    }

    #[test]
    fn serialize_byte_string() {
        //given
        let input = BencodeValue::ByteString("spam".as_bytes());

        //when
        let result = to_bytes(&input);

        //then
        assert_eq!(result, "4:spam".as_bytes().to_vec())
    }

    #[test]
    fn serialize_list() {
        //given
        let input = BencodeValue::List(vec![BencodeValue::ByteString("spam".as_bytes()), BencodeValue::ByteString("foo".as_bytes())]);

        //when
        let result = to_bytes(&input);

        //then
        assert_eq!(result, "l4:spam3:fooe".as_bytes().to_vec())
    }

    #[test]
    fn serialize_dict() {
        //given
        let key1 = "bar".as_bytes();
        let val1 = BencodeValue::ByteString("spam".as_bytes());

        let key2 = "foo".as_bytes();
        let val2 = BencodeValue::Integer(42i64);

        let mut map = IndexMap::new();
        map.insert(key1, val1);
        map.insert(key2, val2);

        let input = BencodeValue::Dict(map);

        //when
        let result = to_bytes(&input);

        //then
        assert_eq!(result, "d3:bar4:spam3:fooi42ee".as_bytes().to_vec())
    }

    #[test]
    fn serialize_complex() -> Result<(), Box<dyn Error>> {
        //given
        let bytes = include_bytes!("../resources/archlinux-2020.06.01-x86_64.iso.torrent");
        let (_, input) = from_bytes(bytes)?;

        //when
        let result = to_bytes(&input);

        //then
        assert_eq!(bytes.to_vec(), result);
        Ok(())
    }
}