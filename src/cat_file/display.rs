use super::super::compression;
use std::str;
use std::{fs::File, io::Read, path::Path};

struct Content {
    hash: String,
    decoded: Vec<u8>,
    file_type: String,
    size: u32,
}

impl Content {
    pub fn new(hash: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
            decoded: Vec::new(),
            file_type: String::new(),
            size: 0,
        }
    }
    fn path(&self) -> Result<String, String> {
        let (sub_dir, filename) = self.hash.split_at(2);
        let path_str = format!(".git/objects/{}/{}", sub_dir, filename);
        let path = Path::new(&path_str);
        if !path.exists() {
            return Err(format!("specified object {} does not found", self.hash).to_string());
        }
        Ok(path_str)
    }
    fn decode(&mut self) -> Result<Vec<u8>, String> {
        if !self.decoded.is_empty() {
            return Ok(self.decoded.clone());
        }

        let path_str = self.path()?;
        let path = Path::new(&path_str);
        let mut buffer = Vec::new();
        match File::open(path) {
            Ok(mut file) => {
                let _ = file.read_to_end(&mut buffer).unwrap();
            }
            Err(_) => return Err("file open error".to_string()),
        }
        compression::zlib::decompress(&buffer, &mut self.decoded).unwrap();
        Ok(self.decoded.clone())
    }
    fn to_string(&self) -> String {
        String::from_utf8(self.decoded.to_vec()).unwrap()
    }
    pub fn list(&mut self) -> Result<(), String> {
        if self.decoded.is_empty() {
            let _ = self.decode().expect("Failed to decode tree object");
        }
        let contents: Vec<&[u8]> = self.decoded.splitn(2, |ch| *ch == b'\x00').collect();
        let (obj_type, size) = std::str::from_utf8(contents[0])
            .unwrap()
            .split_once(" ")
            .unwrap();
        if obj_type != "tree" {
            return Err(format!("Invalid object type: {}", self.file_type));
        }
        self.size = size.parse::<u32>().unwrap();
        let mut entry = contents[1].to_vec();
        while !entry.is_empty() {
            let filemode_pos = entry.iter().position(|&ch| ch == b' ').unwrap();
            let filemode = String::from_utf8(entry.drain(..filemode_pos).collect::<Vec<u8>>())
                .unwrap()
                .parse::<u64>()
                .unwrap();
            let filename_pos = entry.iter().position(|&ch| ch == b'\x00').unwrap() + 1; // NULL文字を含めて良い
            let filename =
                String::from_utf8(entry.drain(..filename_pos).collect::<Vec<u8>>()).unwrap();
            let filehash_pos = 20;
            let filehash = entry
                .drain(..filehash_pos)
                .collect::<Vec<u8>>()
                .iter()
                .take(20)
                .map(|x| format!("{:02x}", x))
                .collect::<String>();
            // entry = entry.drain(..filehash_pos).collect();
            println!("{:06} {} {}\t{}", filemode, obj_type, filehash, filename);
        }

        Ok(())
    }
    pub fn data(&mut self) -> Result<Vec<String>, String> {
        if self.decoded.is_empty() {
            let _ = self.decode()?;
        }
        let decoded_str = self.to_string();
        let data: Vec<&str> = decoded_str.split("\0").collect();
        Ok(data.into_iter().map(String::from).collect())
    }
    pub fn object_type(&mut self) -> Result<String, String> {
        if self.file_type.len() > 0 {
            return Ok(self.file_type.clone());
        }
        if self.decoded.is_empty() {
            let _ = self.decode()?;
        }
        let types_buff: Vec<&[u8]> = self.decoded.split(|ch| *ch == b' ').collect();
        self.file_type = String::from_utf8(types_buff[0].to_vec()).unwrap();
        Ok(self.file_type.clone())
    }
}

pub fn contents(hash: &str) -> Result<(), String> {
    let mut content = Content::new(hash);

    let types = content.object_type()?;
    match types.as_str() {
        "commit" => {
            println!("commit object");
            let data = content.data()?;
            println!("{}", data[1]);
        }
        "tree" => {
            println!("tree object");
            let _ = content.list()?;
        }
        "blob" => {
            println!("blob object");
            let data = content.data()?;
            println!("{}", data[1]);
        }
        _ => {
            return Err("unknown git object".to_string());
        }
    }
    Ok(())
}

pub fn types(hash: &str) -> Result<(), String> {
    let mut content = Content::new(hash);
    let types = content.object_type()?;
    println!("{} object", types);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contents_constructor() {
        let contents = Content::new("test-hash");
        assert_eq!(contents.hash, "test-hash");
    }
    #[test]
    fn test_check_path_if_exists() -> Result<(), String> {
        let contents = Content::new("a605e75b0350483029ac7d96c1038ac128732f63");
        let path = contents.path()?;
        assert_eq!(
            path,
            ".git/objects/a6/05e75b0350483029ac7d96c1038ac128732f63"
        );
        Ok(())
    }
    #[test]
    #[should_panic]
    fn test_check_path_does_not_exists() {
        let contents = Content::new("hogefuga");
        let _ = contents.path().unwrap();
    }
    #[test]
    fn test_decode_blob() -> Result<(), String> {
        let mut contents = Content::new("196e176ded130852dddb034af1b92bce178558c9"); // .gitignore
        let decoded = contents.decode()?;
        assert!(!decoded.is_empty());
        Ok(())
    }
    #[test]
    fn test_decode_tree() -> Result<(), String> {
        let mut contents = Content::new("cd6b39ce605837005418cab9a4b1faeeefa464ca"); // src
        let decoded = contents.decode()?;
        assert!(!decoded.is_empty());
        Ok(())
    }
    #[test]
    fn test_types_commit() {
        let mut contents = Content::new("8d0e1910e145c51c5c5d6df1b3a19261913ad7cc"); // develop commit
        let types = contents.object_type().unwrap();
        assert_eq!(types, "commit");
    }
    #[test]
    fn test_types_tree() {
        let mut contents = Content::new("cd6b39ce605837005418cab9a4b1faeeefa464ca"); // src
        let types = contents.object_type().unwrap();
        assert_eq!(types, "tree");
    }
    #[test]
    fn test_types_blob() {
        let mut contents = Content::new("37dc934f93b32f0f5901cfa451c08d06756d8f8d"); // Cargo.toml
        let types = contents.object_type().unwrap();
        assert_eq!(types, "blob");
    }
}
