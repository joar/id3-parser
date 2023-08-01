use std::fs::File;
use std::path::PathBuf;

use clap::Parser;

use id3::Header;

mod error;
mod id3;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    input_file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let reader = File::open(cli.input_file).unwrap();
    let header = Header::read(reader)?;
    println!("id3: {:?}", header);
    Ok(())
}

mod tests {}
