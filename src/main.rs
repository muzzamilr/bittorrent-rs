mod parser;
mod peer_connection;
mod result;
mod torrent;
mod tracker;

use crate::{
    parser::{decode, ValueToString},
    peer_connection::PeerConnection,
    result::Result,
    torrent::Torrent,
    tracker::TrackerRequest,
};

use std::{env, fs};

const PEER_ID: &str = "00112233445566778899";

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
        // let peer_id = handshake(&meta_data.info, ip)?;
        let connectaion_resp =
            PeerConnection::new(ip)?.handshake(meta_data.info.get_hash()?.to_vec(), PEER_ID)?;

        let peer_id = connectaion_resp
            .info_hash
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<String>>()
            .join("");

        println!("Peer ID: {}", peer_id);
    } else {
        println!("unknown command: {}", args[1])
    }
    Ok(())
}
