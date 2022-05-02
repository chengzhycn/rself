use std::io::{self, BufWriter, Write};

use clap::Parser;
use rself::elf;

#[derive(Parser, Debug)]
#[clap(author,  version, about, long_about = None)]
struct Args {
    /// Display the ELF file header
    #[clap(short = 'h', long)]
    file_header: bool,

    /// Display the program headers
    #[clap(short, long)]
    program_headers: bool,
}

fn main() -> io::Result<()> {
    let _args = Args::parse();

    // let mut f = File::open("main")?;
    let mut buffer = BufWriter::new(io::stdout());

    // let hdr = elf::Elf64Ehdr::from_file(&mut f)?;

    // println!("{}", hdr);

    let mut elf = elf::Elf::new("main");

    elf.to_str(&mut buffer)?;

    // hdr.to_string(&mut buffer);

    buffer.flush()?;

    Ok(())
}
