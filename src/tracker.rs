use super::{result::Result, torrent::TorrentInfo};
use serde::{Deserialize, Serialize};
use serde_bencode::{self, value::Value};

pub struct TrackerRequest {
    pub info_hash: String,
    pub peer_id: String,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub compact: u8,
}

impl TrackerRequest {
    pub fn new(data: &TorrentInfo) -> Result<TrackerRequest> {
        Ok(TrackerRequest {
            info_hash: data.encode_url_hash()?,
            peer_id: "00112233445566778899".to_string(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: data.length as u64,
            compact: 1,
        })
    }

    pub fn fetch_info(&self, tracker_url: String) -> Result<TrackerResponse> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}?info_hash={}", tracker_url, self.info_hash);
        let req = client
            .get(url)
            .query(&[("peer_id", &self.peer_id)])
            .query(&[("port", self.port)])
            .query(&[("uploaded", self.uploaded)])
            .query(&[("downloaded", self.downloaded)])
            .query(&[("left", self.left)])
            .query(&[("compact", self.compact)])
            .build()?;
        let res = client.execute(req)?;

        Ok(serde_bencode::from_bytes(res.bytes()?.as_ref())?)
    }
}

#[derive(Debug, Deserialize)]
pub struct TrackerResponse {
    pub interval: u64,
    #[serde(with = "serde_bytes")]
    pub peers: Vec<u8>,
}

impl TrackerResponse {
    pub fn from_peers(&self) -> Result<Vec<String>> {
        Ok(self
            .peers
            .chunks(6)
            .map(|chuck| {
                let ip = chuck[0..4]
                    .iter()
                    .map(|b| format!("{}", b))
                    .collect::<Vec<String>>()
                    .join(".");
                let port = u16::from_be_bytes([chuck[4], chuck[5]]);
                format!("{}:{}", ip, port)
            })
            .collect())
    }
}
