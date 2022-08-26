use transform::{RegexTransforms};
use std::{io, io::prelude::*};
use std::path::Path;
use clap::Parser;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args{
    // The file to load the regular expressions 
    // and their transforms.
    #[clap(short, long, value_parser)]
    transforms: String,
}

pub fn main() -> Result<()> {
    let args = Args::parse();
    match RegexTransforms::load(Path::new(&args.transforms)){
        Ok(transforms) => {
            for line in io::stdin().lock().lines() {
                println!("{}", transforms.transform_file_path(line.unwrap().trim_end()));
            }
            Ok(())
        },
        Err(e) => {
            eprintln!("{}", e);
            Err(Box::new(e))
        }
    }
}
