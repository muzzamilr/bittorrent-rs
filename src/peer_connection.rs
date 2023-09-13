use super::result::Result;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub enum PeerMessages {
    CHOKE = 0,
    UNCHOKE = 1,
    INTERESTED = 2,
    NOTINTERESTED = 3,
    HAVE = 4,
    BITFIELD = 5,
    REQUEST = 6,
    PIECE = 7,
    CANCEL = 8,
}

pub struct PeerConnectionResponse {
    pub info_hash: Vec<u8>,
    pub peer_id: Vec<u8>,
}

pub struct PeerConnection {
    tcp_stream: TcpStream,
}

impl PeerConnection {
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
        let peer_id = res_buf[48..68]
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<String>>()
            .join("");
        Ok(PeerConnectionResponse {
            info_hash: res_buf[28..48].to_vec(),
            peer_id: res_buf[48..68].to_vec(),
        })
    }

    // pub fn send_interested()
}
