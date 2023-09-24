use super::super::compression;
use std::str;
use std::{fs::File, io::Read, path::Path};

struct Content {
    hash: String,
    decoded: Vec<u8>,
    file_type: String,
}

impl Content {
    pub fn new(hash: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
            decoded: Vec::new(),
            file_type: String::new(),
        }
    }
    fn check_path(&self) -> Result<String, String> {
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

        let path_str = self.check_path()?;
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
    pub fn get_data(&mut self) -> Result<Vec<String>, String> {
        if self.decoded.is_empty() {
            let _ = self.decode()?;
        }
        let decoded_str = self.to_string();
        let data: Vec<&str> = decoded_str.split("\0").collect();
        let file_type_split: Vec<&str> = data[0].split(" ").collect();
        let file_type = file_type_split[0];
        self.file_type = file_type.to_string();
        Ok(data.into_iter().map(String::from).collect())
    }
    pub fn get_type(&mut self) -> Result<String, String> {
        if self.file_type.len() > 0 {
            return Ok(self.file_type.clone());
        }
        let _ = self.get_data()?;
        Ok(self.file_type.clone())
    }
}

pub fn contents(hash: &str) -> Result<(), String> {
    let mut content = Content::new(hash);

    let types = content.get_type()?;
    let data = content.get_data()?;
    match types.as_str() {
        "commit" => {
            println!("commit object");
            println!("{}", data[1]);
        }
        "tree" => {
            println!("commit object");
            println!("{}", data[1]);
        }
        "blob" => {
            println!("commit object");
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
    let types = content.get_type()?;
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
        let path = contents.check_path()?;
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
        let _ = contents.check_path().unwrap();
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
}
