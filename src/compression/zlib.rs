use std::io::Read;

use flate2::read::ZlibDecoder;

pub fn decompress(data: &[u8]) -> String {
    let mut decoder = ZlibDecoder::new(data);
    let mut s = String::new();
    decoder.read_to_string(&mut s).unwrap();
    s
}
