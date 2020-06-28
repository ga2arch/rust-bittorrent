use crate::bencode;
use crate::bencode::BencodeValue;
use serde::private::ser::constrain;

#[derive(Debug, PartialEq)]
pub enum TorrentError {
    InvalidInput
}

#[derive(Debug, PartialEq)]
struct BencodeTorrent {
    pub announce: String
}

impl BencodeTorrent {
    pub fn from_bytes(bytes: &[u8]) -> Result<BencodeTorrent, TorrentError>  {
        let parsed = bencode::from_bytes(bytes)
            .map(|parsed| parsed.1)
            .map_err(|err| TorrentError::InvalidInput)?;

        let announce_key = "announce".as_bytes();

        match parsed {
            BencodeValue::Dict(dict) => {
                let mut torrent = BencodeTorrent { announce: "".to_string() };
                for (key, value) in dict.iter() {
                    match (key, value) {
                        (BencodeValue::ByteString(bkey), BencodeValue::ByteString(bvalue)) if bkey == &announce_key => {
                            torrent.announce = std::str::from_utf8(bvalue).unwrap().to_string();
                        }
                        _ => {}
                    }
                }
                Ok(torrent)
            },

            _ => Err(TorrentError::InvalidInput)?
        }
    }
}

#[cfg(test)]
mod test {
    use crate::torrent::BencodeTorrent;

    #[test]
    fn parse_torrent() {
        //given
        let input = include_bytes!("../resources/archlinux-2020.06.01-x86_64.iso.torrent");

        //when
        let torrent = BencodeTorrent::from_bytes(input);

        //then
        assert_eq!(torrent, Ok(BencodeTorrent {
            announce: "http://tracker.archlinux.org:6969/announce".to_string()
        }));
    }
}