use crate::bencode;
use crate::bencode::BencodeValue;
use serde::private::ser::constrain;
use nom::lib::std::collections::HashMap;
use nom::lib::std::collections::hash_map::RandomState;

#[derive(Debug, PartialEq)]
pub enum TorrentError {
    InvalidInput
}

#[derive(Debug, PartialEq)]
struct BencodeTorrent {
    pub announce: String,
    pub name: String,
    pub length: i64,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
}

impl BencodeTorrent {
    pub fn from_bytes(bytes: &[u8]) -> Result<BencodeTorrent, TorrentError> {
        static ANNOUNCE_KEY: &'static [u8] = "announce".as_bytes();
        static INFO_KEY: &'static [u8] = "info".as_bytes();
        static NAME_KEY: &'static [u8] = "name".as_bytes();
        static LENGTH_KEY: &'static [u8] = "length".as_bytes();
        static PIECE_LENGTH_KEY: &'static [u8] = "piece length".as_bytes();
        static PIECES_KEY: &'static [u8] = "pieces".as_bytes();

        let parsed = bencode::from_bytes(bytes)
            .map(|parsed| parsed.1)
            .map_err(|err| TorrentError::InvalidInput)?;

        if_chain! {
            if let BencodeValue::Dict(dict) = parsed;
            if let BencodeValue::ByteString(announce) = get_key(&dict, ANNOUNCE_KEY)?;
            if let BencodeValue::Dict(info_dict) =  get_key(&dict, INFO_KEY)?;
            if let BencodeValue::ByteString(name) = get_key(&info_dict, NAME_KEY)?;
            if let BencodeValue::Integer(length) = get_key(&info_dict, LENGTH_KEY)?;
            if let BencodeValue::Integer(piece_length) = get_key(&info_dict, PIECE_LENGTH_KEY)?;
            if let BencodeValue::ByteString(pieces) = get_key(&info_dict, PIECES_KEY)?;

            then {
                Ok(BencodeTorrent {
                    announce: std::str::from_utf8(announce).unwrap().to_string(),
                    name: std::str::from_utf8(name).unwrap().to_string(),
                    length: *length,
                    piece_length: *piece_length,
                    pieces: pieces.to_vec() })

            } else {
                Err(TorrentError::InvalidInput)?
            }
        }
    }
}

fn get_key<'a>(dict: &'a HashMap<&[u8], BencodeValue<'a>>, key: &'static [u8]) -> Result<&'a BencodeValue<'a>, TorrentError> {
    dict.get(key).ok_or(TorrentError::InvalidInput)
}

#[cfg(test)]
mod test {
    use crate::torrent::{BencodeTorrent, TorrentError};

    #[test]
    fn parse_torrent() -> Result<(), TorrentError> {
        //given
        let input = include_bytes!("../resources/archlinux-2020.06.01-x86_64.iso.torrent");

        //when
        let torrent = BencodeTorrent::from_bytes(input)?;

        //then
        assert_eq!(torrent.announce, "http://tracker.archlinux.org:6969/announce".to_string());
        assert_eq!(torrent.name, "archlinux-2020.06.01-x86_64.iso".to_string());
        assert_eq!(torrent.length, 694157312);
        assert_eq!(torrent.piece_length, 524288);

        Ok(())
    }
}