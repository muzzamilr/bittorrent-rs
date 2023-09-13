use std::{env, fs, net::TcpStream};
use torrent_lib::{
    parser::{decode, ValueToString},
    result::Result,
    torrent::{Torrent, TorrentInfo},
    tracker::{TrackerRequest, TrackerResponse},
};

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
    } else if command == "handshake" {
        let path = fs::read(&args[2])?;
        let meta_data = Torrent::from_file(path)?;
        let ip = &args[3];
        println!("{:?} {:?}", meta_data, ip);
    } else {
        println!("unknown command: {}", args[1])
    }
    Ok(())
}
