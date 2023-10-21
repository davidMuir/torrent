use anyhow::Result;
use bencode::*;

mod bencode;
mod torrent_file;

fn main() -> Result<()> {
    let torrent = include_bytes!("./big-buck-bunny.torrent");

    let parsed = decode_bencoded(torrent)?;

    println!("{parsed:#?}");

    Ok(())
}
