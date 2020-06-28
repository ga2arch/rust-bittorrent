use crate::torrent::{Torrent, AnnounceUrl};
use url::{Url, ParseError};
use std::error::Error;
use url::form_urlencoded::byte_serialize;
use std::net::Ipv4Addr;
use crate::bencode;
use crate::bencode::BencodeValue;
use core::fmt;
use std::fmt::Formatter;
use indexmap::map::IndexMap;
use std::io::Cursor;

#[derive(Debug, PartialEq)]
pub enum ClientError {
    TrackerError
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::TrackerError => write!(f, "{}", "tracker error")
        }
    }
}

impl Error for ClientError {}

#[derive(Debug, PartialEq)]
pub struct PeerId(pub String);

impl PeerId {
    pub fn default() -> PeerId {
        PeerId("-TR2940-k8hj0wgej6ch".to_string())
    }
}


#[derive(Debug, PartialEq)]
pub struct TrackerUrl(pub String);

impl TrackerUrl {
    pub fn from(torrent: &Torrent, peer_id: &PeerId, port: u32) -> Result<TrackerUrl, Box<dyn Error>> {
        let mut url = Url::parse(torrent.announce.0.as_str())?;
        let mut query = format!("peer_id={peer_id}&port={port}&uploaded=0&downloaded=0&compact=1&left={left}",
                            peer_id = peer_id.0,
                            port = port,
                            left = torrent.length);
        query.extend("&info_hash=".chars());
        query.extend(byte_serialize(torrent.info_hash.0.as_slice()));
        url.set_query(Some(query.as_str()));
        Ok(TrackerUrl(url.to_string()))
    }
}

#[derive(Debug, PartialEq)]
pub struct Peer {
    pub ip: Ipv4Addr,
    pub port: u16
}

impl Peer {
    pub fn from_bytes(bs: &[u8]) -> Peer {
        Peer {
            ip: Ipv4Addr::new(bs[0],bs[1],bs[2],bs[3]),
            port: u16::from_be_bytes([bs[4], bs[5]])
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TrackerResponse {
    pub interval: u64,
    pub peers: Vec<Peer>
}

pub async fn query_tracker(tracker_url: &TrackerUrl) -> Result<TrackerResponse, Box<dyn Error>> {
    static INTERVAL_KEY: &'static [u8] = "interval".as_bytes();
    static PEERS_KEY: &'static [u8] = "peers".as_bytes();

    let response = reqwest::get(tracker_url.0.as_str()).await?.bytes().await?;
    let (_, parsed) = bencode::from_bytes(response.as_ref())
        .map_err(|err| {
            ClientError::TrackerError
        })?;
    if_chain! {
        if let BencodeValue::Dict(dict) = parsed;
        if let BencodeValue::Integer(interval) = get_key(&dict, INTERVAL_KEY)?;
        if let BencodeValue::ByteString(peers) = get_key(&dict, PEERS_KEY)?;

        then {
            Ok(TrackerResponse {
                interval: (*interval) as u64,
                peers: build_peers(peers)
            })
        } else {
            Err(ClientError::TrackerError)?
        }
    }
}

fn build_peers(bs: &[u8]) -> Vec<Peer> {
    bs.chunks(6)
        .map(Peer::from_bytes)
        .collect::<Vec<Peer>>()
}

fn get_key<'a>(dict: &'a IndexMap<&[u8], BencodeValue<'a>>, key: &'static [u8]) -> Result<&'a BencodeValue<'a>, ClientError> {
    dict.get(key).ok_or(ClientError::TrackerError)
}

#[cfg(test)]
mod test {
    use crate::torrent::Torrent;
    use std::error::Error;
    use crate::client::{TrackerUrl, PeerId, query_tracker};

    #[test]
    fn create_tracker_url() -> Result<(), Box<dyn Error>> {
        //given
        let torrent = Torrent::from_bytes(include_bytes!("../resources/archlinux-2020.06.01-x86_64.iso.torrent"))?;
        let peer_id = PeerId::default();
        let port = 6881;

        //when
        let url = TrackerUrl::from(&torrent, &peer_id, port)?;

        //then
        assert_eq!(url.0, "http://tracker.archlinux.org:6969/announce?\
        peer_id=-TR2940-k8hj0wgej6ch\
        &port=6881\
        &uploaded=0\
        &downloaded=0\
        &compact=1\
        &left=694157312\
        &info_hash=%E7%9D%1F%AC%0E%60Y%8B%F0%F1%134%87%85-%81%CFql%ED");
        Ok(())
    }

    #[tokio::test]
    async fn try_query_tracker() -> Result<(), Box<dyn Error>> {
        //given
        let torrent = Torrent::from_bytes(include_bytes!("../resources/archlinux-2020.06.01-x86_64.iso.torrent"))?;
        let peer_id = PeerId::default();
        let port = 6881;

        //when
        let url = TrackerUrl::from(&torrent, &peer_id, port)?;
        let response = query_tracker(&url).await?;

        //then
        println!("{:?}", response);
        Ok(())
    }
}