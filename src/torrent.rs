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
struct Torrent {
    pub announce: String,
    pub name: String,
    pub length: i64,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
}

impl Torrent {
    pub fn from_bytes(bytes: &[u8]) -> Result<Torrent, TorrentError> {
        let parsed = bencode::from_bytes(bytes)
            .map(|parsed| parsed.1)
            .map_err(|err| TorrentError::InvalidInput)?;

        let announce_key = "announce".as_bytes();
        let info_key = "info".as_bytes();
        let name_key = "name".as_bytes();
        let length_key = "length".as_bytes();
        let piece_length_key = "piece length".as_bytes();
        let pieces_key = "pieces".as_bytes();

        match parsed {
            BencodeValue::Dict(dict) => {
                let mut torrent = Torrent { announce: "".to_string(), name: "".to_string(), length: 0, piece_length: 0, pieces: vec![] };

                for (key, value) in dict.iter() {
                    match (key, value) {
                        (BencodeValue::ByteString(bkey), BencodeValue::ByteString(bvalue)) if bkey == &announce_key =>
                            torrent.announce = std::str::from_utf8(bvalue).unwrap().to_string(),

                        (BencodeValue::ByteString(bkey), BencodeValue::Dict(bdict)) if bkey == &info_key => {
                            for (ikey, ivalue) in bdict.iter() {
                                match (ikey, ivalue) {
                                    (BencodeValue::ByteString(k), BencodeValue::ByteString(v)) if k == &name_key =>
                                        torrent.name = std::str::from_utf8(v).unwrap().to_string(),

                                    (BencodeValue::ByteString(k), BencodeValue::Integer(v)) if k == &length_key =>
                                        torrent.length = *v,

                                    (BencodeValue::ByteString(k), BencodeValue::Integer(v)) if k == &piece_length_key =>
                                        torrent.piece_length = *v,

                                    (BencodeValue::ByteString(k), BencodeValue::ByteString(v)) if k == &pieces_key =>
                                        torrent.pieces = v.to_vec(),
                                    _ => {}
                                }
                            }
                        }

                        _ => {}
                    }
                }
                Ok(torrent)
            }

            _ => Err(TorrentError::InvalidInput)?
        }
    }
}

#[cfg(test)]
mod test {
    use crate::torrent::{Torrent, TorrentError};

    #[test]
    fn parse_torrent() -> Result<(), TorrentError> {
        //given
        let input = include_bytes!("../resources/archlinux-2020.06.01-x86_64.iso.torrent");

        //when
        let torrent = Torrent::from_bytes(input)?;

        //then
        assert_eq!(torrent.announce, "http://tracker.archlinux.org:6969/announce".to_string());
        assert_eq!(torrent.name, "archlinux-2020.06.01-x86_64.iso".to_string());
        assert_eq!(torrent.length, 694157312);
        assert_eq!(torrent.piece_length, 524288);

        Ok(())
    }
}