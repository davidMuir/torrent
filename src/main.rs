use anyhow::Result;
use bencode::*;
use std::env;

mod bencode;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded(encoded_value)?;

        println!("{decoded_value:?}");
    } else {
        println!("unknown command: {}", args[1])
    }

    Ok(())
}
