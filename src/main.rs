use serde::{Deserialize, Serialize};
use serde_bencode::{self, value::Value};
use sha1::{Digest, Sha1};
use std::{env, fs};

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
    fn hash(&self) -> String {
        let bytes = serde_bencode::to_bytes(self).unwrap();
        let mut hash = Sha1::new();
        hash.update(bytes);
        format!("{:x}", hash.finalize())
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Torrent {
    announce: String,
    info: TorrentInfo,
}

#[allow(dead_code)]

fn decode(encoded_value: &str) -> Value {
    return serde_bencode::from_str::<Value>(encoded_value).unwrap();
}

trait ValueToString {
    fn to_string(&self) -> String;
}

impl ValueToString for Value {
    fn to_string(&self) -> String {
        return match self {
            Value::Int(i) => i.to_string(),
            Value::List(l) => format!(
                "[{}]",
                l.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Value::Bytes(b) => format!("{:?}", String::from_utf8(b.clone()).unwrap()),
            Value::Dict(d) => {
                let mut result: Vec<String> = Vec::new();
                for (key, value) in d {
                    let key_str = String::from_utf8_lossy(key).to_string();
                    result.push(format!("\"{}\":{}", key_str, value.to_string()));
                }
                result.sort();
                format!("{{{}}}", result.join(","))
            }
        };
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let value = format!("{}", decode(encoded_value).to_string());
        println!("{}", value);
    } else if command == "info" {
        let path = fs::read(&args[2]).unwrap();
        let meta_data: Torrent = serde_bencode::from_bytes(&path).unwrap();
        println!("Tracker URL: {}", meta_data.announce);
        println!("Length: {}", meta_data.info.length);
        println!("Info Hash: {}", meta_data.info.hash());
    } else {
        println!("unknown command: {}", args[1])
    }
}
