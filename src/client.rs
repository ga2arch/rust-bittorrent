use crate::torrent::{Torrent, AnnounceUrl};
use url::{Url, ParseError};
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct PeerId(pub String);

#[derive(Debug, PartialEq)]
pub struct TrackerUrl(pub String);

impl TrackerUrl {
    pub fn from(torrent: &Torrent, peer_id: &PeerId, port: u32) -> Result<TrackerUrl, Box<dyn Error>> {
        let mut url = Url::parse(torrent.announce.0.as_str())?;
        let query = format!("info_hash={info_hash}&peer_id={peer_id}&port={port}&uploaded=0&downloaded=0&compact=1&left={left}",
                            info_hash = torrent.info_hash.to_string(),
                            peer_id = peer_id.0,
                            port = port,
                            left = torrent.length);

        url.set_query(Some(query.as_str()));
        Ok(TrackerUrl(url.to_string()))
    }
}

#[cfg(test)]
mod test {
    use crate::torrent::Torrent;
    use std::error::Error;
    use crate::client::{TrackerUrl, PeerId};

    #[test]
    fn create_tracker_url() -> Result<(), Box<dyn Error>> {
        //given
        let torrent = Torrent::from_bytes(include_bytes!("../resources/archlinux-2020.06.01-x86_64.iso.torrent"))?;
        let peer_id = PeerId("GA2".to_string());
        let port = 6881;

        //when
        let url = TrackerUrl::from(&torrent, &peer_id, port)?;

        //then
        assert_eq!(url.0, "http://tracker.archlinux.org:6969/announce?\
        info_hash=e79d1fac0e60598bf0f1133487852d81cf716ced\
        &peer_id=GA2\
        &port=6881\
        &uploaded=0\
        &downloaded=0\
        &compact=1\
        &left=694157312");
        Ok(())
    }
}