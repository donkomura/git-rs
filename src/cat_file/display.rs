use super::super::compression;
use std::{fs::File, io::Read, path::Path};

struct Content {
    hash: String,
    decoded: String,
    file_type: String,
}

impl Content {
    pub fn new(hash: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
            decoded: String::new(),
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
    fn decode(&mut self) -> Result<String, String> {
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
        self.decoded = compression::zlib::decompress(&buffer);
        Ok(self.decoded.clone())
    }
    pub fn get_data(&mut self) -> Result<Vec<&str>, String> {
        if self.decoded.is_empty() {
            let _ = self.decode()?;
        }
        let data: Vec<&str> = self.decoded.split("\0").collect();
        let file_type_split: Vec<&str> = data[0].split(" ").collect();
        let file_type = file_type_split[0];
        self.file_type = file_type.to_string();
        Ok(data)
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
