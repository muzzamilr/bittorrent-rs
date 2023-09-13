use serde::{Deserialize, Serialize};
use serde_bencode::{self};

use super::result::Result;
use sha1::{digest::Output, Digest, Sha1};

#[derive(Debug, Deserialize, Serialize)]
pub struct TorrentInfo {
    pub name: String,
    pub length: usize,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
}

impl TorrentInfo {
    pub fn get_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_bencode::to_bytes(self)?)
    }
    pub fn get_hash(&self) -> Result<Output<Sha1>> {
        let mut hash = Sha1::new();
        hash.update(self.get_bytes()?);
        Ok(hash.finalize())
    }
    pub fn to_hash_string(&self) -> Result<String> {
        Ok(format!("{:x}", self.get_hash()?))
    }
    pub fn encode_url_hash(&self) -> Result<String> {
        Ok(self
            .get_hash()?
            .iter()
            .map(|v| format!("%{:02x}", v))
            .collect::<Vec<String>>()
            .join(""))
    }
    pub fn chunks_to_hex(&self, chunck: &[u8]) -> Result<String> {
        Ok(chunck
            .iter()
            .map(|val| format!("{:02x}", val))
            .collect::<String>())
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Torrent {
    pub announce: String,
    pub info: TorrentInfo,
}

impl Torrent {
    pub fn from_file(path: Vec<u8>) -> Result<Torrent> {
        Ok(serde_bencode::from_bytes(&path)?)
    }
}
