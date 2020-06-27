use nom::{
    IResult,
    Err,
    bytes::complete::{tag, is_not, take},
    sequence::{tuple, delimited},
    multi::{many0},
    branch::alt,
    error::{ErrorKind, ParseError},
    Err::{Failure}};

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

type BencodeParserResult<'a> = IResult<&'a [u8], BencodeValue, BencodeParserError<'a>>;

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

#[derive(Debug, PartialEq)]
pub enum BencodeValue {
    Integer(i64),
    ByteString(Vec<u8>),
    List(Vec<BencodeValue>),
    Dict(Vec<(BencodeValue, BencodeValue)>),
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
    let (input, (len, _)) = tuple((is_not(":"), tag(":")))(input)?;
    let len = bytes_to_i64(len)?;
    if len > 0 {
        let (input, byte_string) = take(len as u64)(input)?;
        Ok((input, BencodeValue::ByteString(byte_string.to_vec())))
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
    let kv_parser = tuple((parse_byte_string, parse));
    let (input, xs): (&[u8], Vec<(BencodeValue, BencodeValue)>) = delimited(tag("d"), many0(kv_parser), tag("e"))(input)?;
    Ok((input, BencodeValue::Dict(xs)))
}


#[cfg(test)]
mod tests {
    use crate::bencode::{BencodeValue, BencodeParserError, from_bytes};
    use nom::Err::{Failure};

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
        assert_eq!(result, Ok(("".as_bytes(), BencodeValue::ByteString("spam".as_bytes().to_vec()))))
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
        let elem1 = BencodeValue::ByteString("spam".as_bytes().to_vec());
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
        let key1 = BencodeValue::ByteString("bar".as_bytes().to_vec());
        let val1 = BencodeValue::ByteString("spam".as_bytes().to_vec());

        let key2 = BencodeValue::ByteString("foo".as_bytes().to_vec());
        let val2 = BencodeValue::Integer(42i64);

        assert_eq!(result, Ok(("".as_bytes(), BencodeValue::Dict(vec![(key1, val1), (key2, val2)]))))
    }
}