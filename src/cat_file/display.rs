use crate::compression;
use std::io;
use std::io::Error;
use std::str;
use std::{fs::File, io::Read, path::Path};

struct Content {
    hash: String,
    decoded: Vec<u8>,
    file_type: String,
    size: usize,
}

struct TreeEntry {
    mode: u64,
    name: String,
    obj_type: String,
    hash: String,
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
    fn path(&self) -> io::Result<String> {
        let (sub_dir, filename) = self.hash.split_at(2);
        let path_str = format!(".git/objects/{}/{}", sub_dir, filename);
        let path = Path::new(&path_str);
        if !path.exists() {
            return Err(Error::new(
                io::ErrorKind::NotFound,
                format!(".git/objects/{}/{}", sub_dir, filename),
            ));
        }
        Ok(path_str)
    }
    fn decode(&mut self) -> Result<Vec<u8>, String> {
        if !self.decoded.is_empty() {
            return Ok(self.decoded.clone());
        }

        let path_str = self.path().expect("path does not found");
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
    fn to_string(&self) -> Result<String, String> {
        Ok(String::from_utf8(self.decoded.to_vec()).expect("convert binary to string"))
    }
    pub fn list(&mut self) -> Result<Vec<TreeEntry>, String> {
        if self.decoded.is_empty() {
            let _ = self.decode().expect("Failed to decode tree object");
        }
        let contents: Vec<&[u8]> = self.decoded.splitn(2, |ch| *ch == b'\x00').collect();
        let obj_type = self.get_object_type()?;
        if obj_type != "tree" {
            return Err(format!("Invalid object type: {}", self.file_type));
        }
        let mut entry = contents[1].to_vec();
        let mut result: Vec<TreeEntry> = vec![];
        while !entry.is_empty() {
            let filemode_pos = entry.iter().position(|&ch| ch == b' ').unwrap() + 1;
            let filemode = String::from_utf8(entry.drain(..filemode_pos).collect::<Vec<u8>>())
                .unwrap()
                .trim()
                .parse::<u64>()
                .unwrap();
            let mut entry_type = "None";
            if filemode == 40000 {
                entry_type = "tree";
            } else if filemode == 100644 {
                entry_type = "blob"
            }
            let filename_pos = entry.iter().position(|&ch| ch == b'\x00').unwrap() + 1;
            let filename = String::from_utf8(entry.drain(..filename_pos).collect::<Vec<u8>>())
                .unwrap()
                .trim_matches(char::from(0))
                .to_string();
            let filehash_pos = 20;
            let filehash = entry
                .drain(..filehash_pos)
                .collect::<Vec<u8>>()
                .iter()
                .take(20)
                .map(|x| format!("{:02x}", x))
                .collect::<String>();
            result.push(TreeEntry {
                mode: filemode,
                name: filename,
                obj_type: entry_type.to_owned(),
                hash: filehash,
            });
        }

        Ok(result)
    }
    fn get_data(&self) -> Result<Vec<String>, String> {
        let decoded_str = self.to_string()?;
        let data: Vec<&str> = decoded_str.split("\0").collect();
        Ok(data.into_iter().map(String::from).collect())
    }
    pub fn data(&mut self) -> Result<Vec<String>, String> {
        if self.decoded.is_empty() {
            let _ = self.decode()?;
        }
        let data = self.get_data()?;
        Ok(data)
    }
    fn get_object_type(&self) -> Result<String, String> {
        if self.file_type.len() > 0 {
            return Ok(self.file_type.clone());
        }
        let types_buff: Vec<&[u8]> = self.decoded.split(|ch| *ch == b' ').collect();
        Ok(String::from_utf8(types_buff[0].to_vec())
            .expect("Convert to string in getting the object type"))
    }
    pub fn object_type(&mut self) -> Result<String, String> {
        if self.decoded.is_empty() {
            let _ = self.decode()?;
        }
        self.file_type = self.get_object_type()?;
        Ok(self.file_type.clone())
    }
    fn get_size(&self) -> Result<usize, String> {
        if self.size != 0 {
            return Ok(self.size);
        }
        let buff: Vec<&[u8]> = self.decoded.splitn(2, |ch| *ch == b'\x00').collect();
        let (_, size) = std::str::from_utf8(buff[0])
            .unwrap()
            .split_once(" ")
            .unwrap();
        let s = size.parse::<usize>().expect("Parse str to usize");
        Ok(s)
    }
    pub fn size(&mut self) -> Result<usize, String> {
        if self.decoded.is_empty() {
            let _ = self.decode()?;
        }
        Ok(self.get_size()?)
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
            let entries = content.list()?;
            for entry in entries {
                println!(
                    "{:06} {} {}\t{}",
                    entry.mode, entry.obj_type, entry.hash, entry.name
                );
            }
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

pub fn size(hash: &str) -> Result<(), String> {
    let mut content = Content::new(hash);
    let size = content.size()?;
    println!("{}", size);
    Ok(())
}

#[cfg(test)]
mod tests {
    use core::panic;
    use rand::{self, Rng};
    use std::{fs, io::Write};

    use super::*;

    fn filename_helper(testname: &str) -> String {
        const TESTFILE_PREFIX: &str = "test_";
        format!("{}{}.txt", TESTFILE_PREFIX, testname)
    }
    fn dirname_helper(testname: &str) -> String {
        const TESTDIR_PREFIX: &str = "test_dir_";
        format!("{}{}", TESTDIR_PREFIX, testname)
    }

    fn hash_helper(testname: &str, types: &str) -> String {
        match types {
            "blob" => {
                let mut file = File::create(filename_helper(testname)).expect("create file");
                let mut content = [0u8; 20];
                rand::thread_rng().fill(&mut content[..]);
                file.write_all(&content).expect("write file");
                let hash = std::process::Command::new("git")
                    .args(["hash-object", "-w", &filename_helper(testname)])
                    .stdout(std::process::Stdio::piped())
                    .output()
                    .unwrap()
                    .stdout;
                String::from_utf8(hash).unwrap().trim().to_owned()
            }
            "tree" => {
                // not implemented
                panic!("not implemented");
            }
            "commit" => {
                // not implemented
                panic!("not implemented");
            }
            _ => {
                panic!("unknown types");
            }
        }
    }
    fn remove_file_helper(path: &str) -> io::Result<()> {
        fs::remove_file(path)?;
        Ok(())
    }
    fn remove_file_from_hash_helper(hash: &str) -> io::Result<()> {
        let (sub_dir, filename) = hash.split_at(2);
        let path = format!(".git/objects/{}/{}", sub_dir, filename);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn test_contents_constructor() {
        let contents = Content::new("test-hash");
        assert_eq!(contents.hash, "test-hash");
    }
    #[test]
    fn test_path_exists() -> Result<(), String> {
        let hash = hash_helper("path_exists", "blob");
        let contents = Content::new(&hash);
        let got = contents.path().unwrap();
        let (dir, filename) = hash.split_at(2);
        assert_eq!(got, format!(".git/objects/{}/{}", dir, filename));
        remove_file_helper(&filename_helper("path_exists")).unwrap();
        remove_file_from_hash_helper(&hash).unwrap();
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
        let hash = hash_helper("decode_blob", "blob");
        let mut contents = Content::new(&hash);
        let decoded = contents.decode()?;
        assert!(!decoded.is_empty());
        remove_file_helper(&filename_helper("decode_blob")).unwrap();
        remove_file_from_hash_helper(&hash).unwrap();
        Ok(())
    }
    #[test]
    fn test_types_blob() {
        let hash = hash_helper("types_blob", "blob");
        let mut contents = Content::new(&hash);
        let types = contents.object_type().unwrap();
        assert_eq!(types, "blob");
        remove_file_helper(&filename_helper("types_blob")).unwrap();
        remove_file_from_hash_helper(&hash).unwrap();
    }
    #[test]
    fn test_size() {
        let hash = hash_helper("size", "blob");
        let mut contents = Content::new(&hash);
        let got = contents.size().unwrap();
        let want_binary = std::process::Command::new("git")
            .args(["cat-file", "-s", &hash])
            .stdout(std::process::Stdio::piped())
            .output()
            .unwrap()
            .stdout;
        let want_str = String::from_utf8(want_binary).unwrap();
        let want = want_str.trim_end().parse::<usize>().unwrap();

        assert_eq!(got, want);

        remove_file_helper(&filename_helper("size")).unwrap();
        remove_file_from_hash_helper(&hash).unwrap();
    }
}
