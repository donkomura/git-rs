use clap::{Parser, Subcommand};
use git_rs::cat_file;
use git_rs::hash_object;

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    arg_required_else_help = true,
)]
struct Options {
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[command(about = "provider content or type information")]
    CatFile {
        #[arg(short = 't', long = "type", help = "show object type")]
        is_types: bool,
        #[arg(short = 'p', long = "hash", help = "show object content")]
        is_hash: bool,
        #[arg(short = 's', long = "size", help = "show the object size identified")]
        is_size: bool,
        #[arg(help = "hash value of the object")]
        hash: String,
    },
    HashObject {
        #[arg(
            short = 'w',
            long = "write",
            help = "write the object into the object database"
        )]
        is_write: bool,
        #[arg(short = 't', long = "type", help = "specify the type")]
        object_type: String,
        #[arg(help = "file to hash")]
        file: String,
    },
}

fn main() {
    let options = Options::parse();

    match options.subcommand {
        Subcommands::CatFile {
            is_types,
            is_hash,
            is_size,
            hash,
        } => {
            if is_hash {
                let _ = cat_file::display::contents(&hash).unwrap();
            } else if is_types {
                let _ = cat_file::display::types(&hash).unwrap();
            } else if is_size {
                let _ = cat_file::display::size(&hash).unwrap();
            } else {
                println!("At least 1 option should be specified. Abort.");
            }
        }
        Subcommands::HashObject {
            is_write,
            object_type,
            file,
        } => {
            if is_write {
                let _ = hash_object::content::write(&object_type, &file).unwrap();
            } else {
                let hash_str = hash_object::content::hash(&object_type, &file).unwrap();
                println!("{}", hash_str);
            }
        }
    }
}
