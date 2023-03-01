use clap::Parser;
use rayon::prelude::*;
use std::{
    fs::{read, read_dir, remove_file, rename, File},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

/// CSV Delimiter Converter.
#[derive(Parser, Debug)]
struct Cli {
    /// File containing the header.
    header_file: String,

    /// Target folder containing files to add header.
    target_folder: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let header = read(PathBuf::from(args.header_file))?;
    let target_folder = args.target_folder;
    run_through_dir(&header, &target_folder)?;

    Ok(())
}

fn run_through_dir(header: &[u8], dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dir = read_dir(&dir_path)?;

    dir.par_bridge().for_each(|file| {
        let file = file.expect("Should be able to access files in folder.");
        prepend_to_file(header, &file.path()).expect("Should be able to prepend to file.");
    });

    Ok(())
}

fn prepend_to_file(header: &[u8], path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if compare_header(header, path) {
        return Ok(());
    }

    let file = File::open(path)?;

    let mut reader = BufReader::new(file);

    let parent = path.parent().unwrap();
    let temp_path = parent.join(path.file_name().unwrap().to_str().unwrap().to_owned() + "_temp_");
    let temp = File::create(&temp_path)?;

    let mut temp_writer = BufWriter::new(temp);

    temp_writer.write_all(header)?;

    let mut buf = [0; 1000];

    loop {
        let n = reader.read(&mut buf)?;
        temp_writer.write_all(&buf[0..n])?;
        buf = [0; 1000];

        if n == 0 {
            break;
        }
    }

    remove_file(path)?;
    rename(PathBuf::from(&temp_path), path)?;

    Ok(())
}

fn compare_header(header: &[u8], path: &PathBuf) -> bool {
    let mut file = File::open(path).expect("lolol");
    let mut buf = vec![0 as u8; header.len()];

    file.read(&mut buf).expect("File should be readable.");

    buf == header
}
