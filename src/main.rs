use serde::{Deserialize, Serialize};
use serde_bencode::{self, value::Value};
use sha1::{Digest, Sha1};
use std::{env, fs};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
struct TorrentInfo {
    name: String,
    length: usize,
    #[serde(rename = "piece length")]
    piece_length: usize,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>,
}

impl TorrentInfo {
    fn to_hash_string(&self) -> Result<String> {
        let bytes = serde_bencode::to_bytes(self)?;
        let mut hash = Sha1::new();
        hash.update(bytes);
        Ok(format!("{:x}", hash.finalize()))
    }
    fn chunks_to_hex(&self, chunck: &[u8]) -> Result<String> {
        Ok(chunck
            .iter()
            .map(|val| format!("{:02x}", val))
            .collect::<String>())
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Torrent {
    announce: String,
    info: TorrentInfo,
}

impl Torrent {
    fn from_file(path: Vec<u8>) -> Result<Torrent> {
        Ok(serde_bencode::from_bytes(&path).unwrap())
    }
}

#[allow(dead_code)]

fn decode(encoded_value: &str) -> Result<Value> {
    Ok(serde_bencode::from_str::<Value>(encoded_value)?)
}

trait ValueToString {
    fn to_string(&self) -> Result<String>;
}

impl ValueToString for Value {
    fn to_string(&self) -> Result<String> {
        return match self {
            Value::Int(i) => Ok(i.to_string()),
            Value::List(l) => Ok(format!(
                "[{}]",
                l.iter()
                    .map(|v| v.to_string())
                    .collect::<Result<Vec<String>>>()?
                    .join(",")
            )),
            Value::Bytes(b) => Ok(format!("{:?}", String::from_utf8(b.clone()).unwrap())),
            Value::Dict(d) => {
                let mut result: Vec<String> = Vec::new();
                for (key, value) in d {
                    let key_str = String::from_utf8_lossy(key).to_string();
                    result.push(format!("\"{}\":{}", key_str, value.to_string()?));
                }
                result.sort();
                Ok(format!("{{{}}}", result.join(",")))
            }
        };
    }
}

fn encode_url(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    let hashed = hasher
        .finalize()
        .iter()
        .map(|b| format!("%{:02x}", b))
        .collect::<Vec<String>>()
        .join("");
    hashed
}

struct TrackerRequest {
    info_hash: String,
    peer_id: String,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    compact: u8,
}

impl TrackerRequest {
    fn new(data: TorrentInfo) -> Result<TrackerRequest> {
        Ok(TrackerRequest {
            info_hash: encode_url(&serde_bencode::to_bytes(&data).unwrap()),
            peer_id: "00112233445566778899".to_string(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: data.length as u64,
            compact: 1,
        })
    }

    fn fetch_info(&self, tracker_url: String) -> Result<TrackerResponse> {
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
            .build()
            .unwrap();
        let res = client.execute(req).unwrap();

        Ok(serde_bencode::from_bytes(res.bytes().unwrap().as_ref()).unwrap())
    }
}

#[derive(Debug, Deserialize)]
struct TrackerResponse {
    interval: u64,
    #[serde(with = "serde_bytes")]
    peers: Vec<u8>,
}

impl TrackerResponse {
    fn from_peers(&self) -> Result<Vec<String>> {
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

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let value = format!("{}", decode(encoded_value)?.to_string()?);
        println!("{}", value);
    } else if command == "info" {
        let path = fs::read(&args[2])?;
        let meta_data: Torrent = Torrent::from_file(path)?;
        println!("Tracker URL: {}", meta_data.announce);
        println!("Length: {}", meta_data.info.length);
        println!("Info Hash: {}", meta_data.info.to_hash_string()?);
        println!("Piece Length: {}", meta_data.info.piece_length);
        println!("Piece Hashes:");
        for chunck in meta_data.info.pieces.chunks(20) {
            let chuck_hex = meta_data.info.chunks_to_hex(chunck)?;
            println!("{}", chuck_hex);
        }
    } else if command == "peers" {
        let path = fs::read(&args[2])?;
        let meta_data: Torrent = Torrent::from_file(path)?;
        let res = TrackerRequest::new(meta_data.info)?.fetch_info(meta_data.announce)?;
        for peer in res.from_peers()? {
            println!("{}", peer);
        }
    } else {
        println!("unknown command: {}", args[1])
    }
    Ok(())
}
