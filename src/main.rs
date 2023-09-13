mod parser;
mod result;
mod torrent;
mod tracker;

use crate::{
    parser::{decode, ValueToString},
    result::Result,
    torrent::{Torrent, TorrentInfo},
    tracker::{TrackerRequest, TrackerResponse},
};

use std::{
    env, fs,
    io::{Read, Write},
    net::TcpStream,
};

fn handshake(meta_info: &TorrentInfo, peer: &str) -> Result<String> {
    let mut handshake: Vec<u8> = Vec::new();
    handshake.push(19);
    handshake.extend_from_slice("BitTorrent protocol".as_bytes());
    handshake.extend_from_slice(&[0 as u8; 8]);
    handshake.append(&mut meta_info.get_hash()?.to_vec());
    handshake.extend_from_slice("00112233445566778899".as_bytes());
    let mut tcp_client = TcpStream::connect(peer)?;
    tcp_client.write_all(&handshake)?;
    let mut res_buf = vec![0 as u8; 68];
    tcp_client.read_exact(&mut res_buf)?;
    let peer_id = handshake[48..68]
        .iter()
        .map(|x| format!("{:x}", x))
        .collect::<Vec<String>>()
        .join("");
    Ok(peer_id)
}

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
        let res = TrackerRequest::new(&meta_data.info)?.fetch_info(meta_data.announce)?;
        for peer in res.from_peers()? {
            println!("{}", peer);
        }
    } else if command == "handshake" {
        let path = fs::read(&args[2])?;
        let meta_data = Torrent::from_file(path)?;
        let ip = if args.len() > 3 {
            args[3].clone()
        } else {
            let res =
                TrackerRequest::new(&meta_data.info)?.fetch_info(meta_data.announce.clone())?;
            res.from_peers()?[0].to_string()
        };
        let peer_id = handshake(&meta_data.info, &ip)?;
        println!("Peer ID: {}", peer_id);
    } else {
        println!("unknown command: {}", args[1])
    }
    Ok(())
}
