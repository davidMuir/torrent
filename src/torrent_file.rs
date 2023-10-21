use crate::bencode::decode_bencoded;
use anyhow::Result;

pub struct TorrentFile<'a> {
    announce: &'a str,
    length: u64,
    piece_length: u64,
    info_hash: [u8; 20],
    piece_hashes: Vec<[u8; 20]>,
}

impl<'a> TorrentFile<'a> {
    fn from_bencode(input: &[u8]) -> Result<Self> {
        let decoded = decode_bencoded(input)?;

        todo!()
    }
}
