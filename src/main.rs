use std::io::{self, BufWriter, Write};

use clap::Parser;
use rself::elf;

#[derive(Parser, Debug)]
#[clap(name = "rself")]
#[clap(author = "chengzhycn <chengzhycn@gmail.com>")]
#[clap(version = "0.1.0")]
#[clap(about = "A tool for parsing ELF file.", long_about = None)]
struct Args {
    /// Display the ELF file header
    #[clap(short = 'h', long)]
    file_header: bool,

    /// Display the program headers
    #[clap(short = 'l', long)]
    program_headers: bool,

    /// Display the sections' header
    #[clap(short = 'S', long)]
    section_headers: bool,

    /// Equivalent to: -h -p -s
    #[clap(short, long)]
    all: bool,

    /// elf-file
    #[clap(required = true)]
    file: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut options = elf::Options {
        file_header: args.file_header,
        program_headers: args.program_headers,
        section_headers: args.section_headers,
    };

    if args.all {
        options.file_header = true;
        options.program_headers = true;
        options.section_headers = true;
    }

    if let Some(file) = args.file.as_deref() {
        let mut buffer = BufWriter::new(io::stdout());
        let mut elf = elf::Elf::new(file, options);

        elf.to_str(&mut buffer)?;
        buffer.flush()?;
    }

    Ok(())
}
