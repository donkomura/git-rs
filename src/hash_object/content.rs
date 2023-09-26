use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::compression::zlib;

pub fn hash(object_type: &str, filename: &str) -> Result<String, String> {
    let path = Path::new(filename);
    if !path.exists() {
        return Err(format!(
            "fatal: pathspec '{}' did not match any files",
            filename
        ));
    }
    let content = std::fs::read_to_string(path).unwrap();
    let format = format!(
        "{} {}\x00{}",
        object_type,
        content.as_bytes().len(),
        content
    );
    let mut hasher = Sha1::new();
    hasher.input_str(&format);
    let hex = hasher.result_str();
    Ok(hex)
}

pub fn write(object_type: &str, filename: &str) -> Result<(), String> {
    let path = Path::new(filename);
    if !path.exists() {
        return Err(format!(
            "fatal: pathspec '{}' did not match any files",
            filename
        ));
    }
    let content = std::fs::read_to_string(path).unwrap();
    let content = format!("blob {}\x00{}", content.as_bytes().len(), content);
    let compressed = zlib::compress(content.as_bytes()).unwrap();
    let hash = hash(object_type, filename)?;
    let (dirname, filename) = hash.split_at(2);
    std::fs::create_dir_all(format!(".git/objects/{}", dirname)).unwrap();
    let obj_path = format!(".git/objects/{}/{}", dirname, filename);
    let mut file = File::create(obj_path).unwrap();
    file.write_all(&compressed).unwrap();

    Ok(())
}

#[test]
fn test_hash() {
    std::fs::write("test_hash.txt", "hoge").unwrap();
    let hash = hash("blob", "test_hash.txt").unwrap();
    assert_eq!(hash, "c2684e0321eedff1890b7690c89726387d2af3ca");
    std::fs::remove_file("test_hash.txt").unwrap();
}

#[test]
fn test_write_cat_file() {
    std::fs::write("test_write_cat_file.txt", "hoge").unwrap();
    write("blob", "test_write_cat_file.txt").unwrap();
    let cat_file = std::process::Command::new("git")
        .args(&["cat-file", "-p", "c2684e0321eedff1890b7690c89726387d2af3ca"])
        .output()
        .unwrap();
    let cat_file = String::from_utf8(cat_file.stdout).unwrap();
    assert_eq!(cat_file, "hoge");
    std::fs::remove_file("test_write_cat_file.txt").unwrap();
    std::fs::remove_file(".git/objects/c2/684e0321eedff1890b7690c89726387d2af3ca").unwrap();
}
