use std::io::{Read, Result, Write};

use flate2::read::ZlibDecoder;

pub fn decompress<'a>(bytes: &'a [u8], buffer: &'a mut Vec<u8>) -> Result<()> {
    let mut decoder = ZlibDecoder::new(&bytes[..]);
    decoder.read_to_end(buffer)?;
    Ok(())
}

pub fn compress<'a>(data: &'a [u8]) -> Result<Vec<u8>> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(&data)?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}
