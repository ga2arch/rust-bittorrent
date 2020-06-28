use crate::bencode;
use crate::bencode::BencodeValue;
use serde::private::ser::constrain;
use nom::lib::std::collections::HashMap;
use nom::lib::std::collections::hash_map::RandomState;
use nom::lib::std::slice::Chunks;
use nom::{FindSubstring, InputTake};
use sha1::{Sha1, Digest};
use hex_literal::hex;
use indexmap::map::IndexMap;
use core::fmt;
use nom::lib::std::fmt::Formatter;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum TorrentError {
    InvalidInput
}

impl fmt::Display for TorrentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TorrentError::InvalidInput => write!(f, "{}", "invalid input")
        }
    }
}

impl Error for TorrentError {}

#[derive(Debug, PartialEq)]
pub struct AnnounceUrl(pub String);

#[derive(Debug, PartialEq)]
pub struct InfoHash(pub Vec<u8>);

impl fmt::Display for InfoHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.as_slice()))
    }
}

#[derive(Debug, PartialEq)]
pub struct Torrent {
    pub announce: AnnounceUrl,
    pub name: String,
    pub length: i64,
    pub piece_length: i64,
    pub info_hash: InfoHash,
    pieces: Vec<u8>,
}

impl Torrent {
    pub fn from_bytes(bytes: &[u8]) -> Result<Torrent, TorrentError> {
        static ANNOUNCE_KEY: &'static [u8] = "announce".as_bytes();
        static INFO_KEY: &'static [u8] = "info".as_bytes();
        static NAME_KEY: &'static [u8] = "name".as_bytes();
        static LENGTH_KEY: &'static [u8] = "length".as_bytes();
        static PIECE_LENGTH_KEY: &'static [u8] = "piece length".as_bytes();
        static PIECES_KEY: &'static [u8] = "pieces".as_bytes();

        let parsed = bencode::from_bytes(bytes)
            .map(|parsed| parsed.1)
            .map_err(|err| TorrentError::InvalidInput)?;

        let sub = bytes.find_substring("4:info").ok_or(TorrentError::InvalidInput)?;

        if_chain! {
            if let BencodeValue::Dict(dict) = parsed;
            if let BencodeValue::ByteString(announce) = get_key(&dict, ANNOUNCE_KEY)?;
            let wrapped_info_dict = get_key(&dict, INFO_KEY)?;
            if let BencodeValue::Dict(info_dict) = wrapped_info_dict;
            if let BencodeValue::ByteString(name) = get_key(&info_dict, NAME_KEY)?;
            if let BencodeValue::Integer(length) = get_key(&info_dict, LENGTH_KEY)?;
            if let BencodeValue::Integer(piece_length) = get_key(&info_dict, PIECE_LENGTH_KEY)?;
            if let BencodeValue::ByteString(pieces) = get_key(&info_dict, PIECES_KEY)?;

            then {
                Ok(Torrent {
                    announce: AnnounceUrl(std::str::from_utf8(announce).unwrap().to_string()),
                    name: std::str::from_utf8(name).unwrap().to_string(),
                    length: *length,
                    piece_length: *piece_length,
                    info_hash: InfoHash(build_info_hash(bencode::to_bytes(&wrapped_info_dict).as_slice())),
                    pieces: pieces.to_vec() })

            } else {
                Err(TorrentError::InvalidInput)?
            }
        }
    }

    pub fn pieces(&self) -> Chunks<'_, u8> {
        self.pieces.chunks(20)
    }
}

fn get_key<'a>(dict: &'a IndexMap<&[u8], BencodeValue<'a>>, key: &'static [u8]) -> Result<&'a BencodeValue<'a>, TorrentError> {
    dict.get(key).ok_or(TorrentError::InvalidInput)
}

fn build_info_hash(info_dict: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(info_dict);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod test {
    use crate::torrent::{Torrent, TorrentError, AnnounceUrl, InfoHash};

    #[test]
    fn parse_torrent() -> Result<(), TorrentError> {
        //given
        let input = include_bytes!("../resources/archlinux-2020.06.01-x86_64.iso.torrent");

        //when
        let torrent = Torrent::from_bytes(input)?;

        //then
        assert_eq!(torrent.announce, AnnounceUrl("http://tracker.archlinux.org:6969/announce".to_string()));
        assert_eq!(torrent.name, "archlinux-2020.06.01-x86_64.iso".to_string());
        assert_eq!(torrent.length, 694157312);
        assert_eq!(torrent.piece_length, 524288);
        assert_eq!(torrent.pieces().count(), (torrent.length / torrent.piece_length) as usize);
        assert_eq!(torrent.info_hash, InfoHash(hex!("e79d1fac0e60598bf0f1133487852d81cf716ced").to_vec()));
        Ok(())
    }

    #[test]
    fn info_hash_to_string() {
        //given+
        let hash = "e79d1fac0e60598bf0f1133487852d81cf716ced";
        let info_hash = InfoHash(hex::decode(hash).unwrap());

        //when
        let result = info_hash.to_string();

        //then
        assert_eq!(result, hash);
    }
}