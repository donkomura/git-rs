use std::io::{Read, Result};

use flate2::read::ZlibDecoder;

pub fn decompress<'a>(bytes: &'a [u8], buffer: &'a mut Vec<u8>) -> Result<()> {
    let mut decoder = ZlibDecoder::new(&bytes[..]);
    decoder.read_to_end(buffer)?;
    Ok(())
}
