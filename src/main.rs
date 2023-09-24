use clap::{Parser, Subcommand};
use git_rs::cat_file;

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
    }
}
