use sha1::{Digest, Sha1};

use crate::torrent::Torrent;

use super::result::Result;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub struct PeerConnectionResponse {
    pub info_hash: Vec<u8>,
    pub peer_id: Vec<u8>,
}

pub struct PeerConnection {
    tcp_stream: TcpStream,
}

impl PeerConnection {
    pub const CHOKE: u8 = 0;
    pub const UNCHOKE: u8 = 1;
    pub const INTERESTED: u8 = 2;
    pub const NOTINTERESTED: u8 = 3;
    pub const HAVE: u8 = 4;
    pub const BITFIELD: u8 = 5;
    pub const REQUEST: u8 = 6;
    pub const PIECE: u8 = 7;
    pub const CANCEL: u8 = 8;

    pub fn new(peer: String) -> Result<Self> {
        Ok(PeerConnection {
            tcp_stream: TcpStream::connect(peer)?,
        })
    }

    pub fn handshake(
        &mut self,
        info_hash: Vec<u8>,
        peer_id: &str,
    ) -> Result<PeerConnectionResponse> {
        let mut handshake: Vec<u8> = Vec::new();
        handshake.push(19 as u8);
        handshake.extend_from_slice("BitTorrent protocol".as_bytes());
        handshake.extend_from_slice(&[0; 8]);
        handshake.extend(info_hash);
        handshake.extend_from_slice(&peer_id.as_bytes());

        self.tcp_stream.write_all(handshake.as_slice())?;
        let mut res_buf = [0; 68];
        self.tcp_stream.read_exact(&mut res_buf)?;

        Ok(PeerConnectionResponse {
            info_hash: res_buf[28..48].to_vec(),
            peer_id: res_buf[48..68].to_vec(),
        })
    }

    pub fn send_message(&mut self, id: u8, payload: Vec<u8>) -> Result<()> {
        let mut message: Vec<u8> = vec![0; 5 + payload.len()];
        let length: u32 = if payload.len() == 0 {
            1
        } else {
            payload.len() as u32
        };
        message[0..4].copy_from_slice(&length.to_be_bytes());
        message[4] = id;
        message[5..].copy_from_slice(&payload);
        self.tcp_stream.write_all(&message)?;
        Ok(())
    }

    pub fn send_interested(&mut self) -> Result<()> {
        self.send_message(PeerConnection::INTERESTED, vec![])?;
        Ok(())
    }

    pub fn send_request(&mut self, index: u32, begin: u32, length: u32) -> Result<()> {
        let mut payload: Vec<u8> = vec![0; 12];
        payload[0..4].copy_from_slice(&index.to_be_bytes());
        payload[4..8].copy_from_slice(&begin.to_be_bytes());
        payload[8..12].copy_from_slice(&length.to_be_bytes());
        self.send_message(PeerConnection::REQUEST, payload)?;
        Ok(())
    }

    // pub fn download_piece(&mut self, meta_data: Torrent, piece_index: u32, path: String) {}
    pub fn download_piece(&mut self, meta: Torrent, piece_index: u32, path: String) -> Result<()> {
        let is_last_piece = piece_index as usize == meta.info.hex_pieces().len() - 1;
        let piece_length = if is_last_piece {
            meta.info.length - (piece_index as usize * meta.info.piece_length)
        } else {
            meta.info.piece_length
        };
        println!("* Piece length: {}", piece_length);
        const CHUNK_SIZE: usize = 16 * 1024;
        let block_count = piece_length / CHUNK_SIZE + (piece_length % CHUNK_SIZE != 0) as usize;
        for i in 0..block_count {
            println!("++ Requesting block {}", i);
            let length = if i == block_count - 1 {
                piece_length - (i * CHUNK_SIZE)
            } else {
                CHUNK_SIZE
            };
            self.send_request(piece_index, (i * CHUNK_SIZE) as u32, length as u32);
        }
        let mut piece_data = vec![0; piece_length];
        for _ in 0..block_count {
            let resp = self.wait(PeerConnection::PIECE)?;
            println!("* Received response of length {}", resp.len());
            let index = u32::from_be_bytes([resp[0], resp[1], resp[2], resp[3]]);
            if index != piece_index {
                println!("index mismatch, expected {}, got {}", &piece_index, index);
                continue;
            }
            let begin = u32::from_be_bytes([resp[4], resp[5], resp[6], resp[7]]) as usize;
            piece_data.splice(begin..begin + resp[8..].len(), resp[8..].iter().cloned());
            println!(
                "-- Received block {} of length {}",
                begin / CHUNK_SIZE,
                resp.len() - 8
            );
        }
        println!("% All pieces received, verifying hash");
        let mut hasher = Sha1::new();
        hasher.update(&piece_data.as_slice());
        let fetched_piece_hash = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join("");
        let piece_hash = meta.info.hex_pieces()[piece_index as usize].clone();
        // println!("% Expected piece hash: {}", &piece_hash);
        // println!("% Received piece hash: {}", &fetched_piece_hash);
        if fetched_piece_hash == piece_hash {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&path)
                .unwrap();
            file.write_all(&piece_data).unwrap();
            println!("Piece {} downloaded to {}.", &piece_index, &path);
        } else {
            println!(
                "% piece hash mismatch, expected {}({}), got {}({})",
                piece_hash,
                piece_hash.len(),
                fetched_piece_hash,
                fetched_piece_hash.len()
            );
        }

        Ok(())
    }

    pub fn wait(&mut self, id: u8) -> Result<Vec<u8>> {
        let mut length_prefix = [0; 4];
        self.tcp_stream.read_exact(&mut length_prefix)?;
        let mut message_id = [0; 1];
        self.tcp_stream.read_exact(&mut message_id)?;
        if (message_id[0] != id) {
            panic!("* Expected message id {}, got {}", id, message_id[0]);
        }
        let resp_size = u32::from_be_bytes(length_prefix) - 1;
        let mut payload = vec![0; resp_size as usize];
        self.tcp_stream.read_exact(&mut payload)?;
        Ok(payload)
    }
}
