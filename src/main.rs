use clap::Parser;
use rayon::prelude::*;
use std::{
    fs::{read, read_dir, remove_file, rename, File},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
    str::from_utf8,
};
use strsim::normalized_levenshtein;

/// CSV Delimiter Converter.
#[derive(Parser, Debug)]
struct Cli {
    /// File containing the header.
    header_file: String,

    /// Target folder containing files to add header.
    target_folder: String,

    /// Regex to match file names that will be considered for the headers.
    #[arg(short, long)]
    regex: Option<String>,

    /// Regex to match file names that will be considered for the headers.
    #[arg(short, long, default_value_t = false)]
    delete: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    // let args = Cli {
    //     header_file: "test_header.txt".to_owned(),
    //     target_folder: "testing".to_owned(),
    //     regex: None,
    //     delete: true,
    // };

    let header = read(PathBuf::from(args.header_file))?;
    let target_folder = args.target_folder;
    run_through_dir(&header, &target_folder, args.regex, args.delete)?;

    Ok(())
}

fn run_through_dir(
    header: &[u8],
    dir_path: &str,
    reg: Option<String>,
    should_delete: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = read_dir(&dir_path)?;

    let reg = if let Some(x) = reg {
        regex::Regex::new(&x)?
    } else {
        regex::Regex::new(r".*")?
    };

    dir.par_bridge().for_each(|file| {
        let file = file.expect("Should be able to access files in folder.");
        if !reg.is_match(
            &file
                .file_name()
                .to_str()
                .expect("Name should be convertible to str."),
        ) {
            return ();
        }

        if should_delete {
            remove_from_file(header, &file.path()).expect("Should be able to delete from file.");
        } else {
            prepend_to_file(header, &file.path()).expect("Should be able to prepend to file.");
        };
    });

    Ok(())
}

fn prepend_to_file(header: &[u8], path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if has_header(header, path) {
        return Ok(());
    }

    let file = File::open(path)?;

    let mut reader = BufReader::new(file);

    let parent = path.parent().unwrap();
    let temp_path = parent.join(path.file_name().unwrap().to_str().unwrap().to_owned() + "_temp_");
    let temp = File::create(&temp_path)?;

    let mut temp_writer = BufWriter::new(temp);

    temp_writer.write_all(header)?;

    let mut buf = [0; 4000];

    loop {
        let n = reader.read(&mut buf)?;
        temp_writer.write_all(&buf[0..n])?;
        buf = [0; 4000];

        if n == 0 {
            break;
        }
    }

    remove_file(path)?;
    rename(PathBuf::from(&temp_path), path)?;

    Ok(())
}

fn remove_from_file(header: &[u8], path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !has_header(header, path) {
        return Ok(());
    }

    let file = File::open(path)?;

    let mut reader = BufReader::new(file);

    let parent = path.parent().unwrap();
    let temp_path = parent.join(path.file_name().unwrap().to_str().unwrap().to_owned() + "_temp_");
    let temp = File::create(&temp_path)?;

    let mut temp_writer = BufWriter::new(temp);

    let mut buf_to_throw_away = vec![0; header.len()];

    reader.read(&mut buf_to_throw_away)?;

    let mut buf = [0; 4000];

    loop {
        let n = reader.read(&mut buf)?;
        temp_writer.write_all(&buf[0..n])?;
        buf = [0; 4000];

        if n == 0 {
            break;
        }
    }

    remove_file(path)?;
    rename(PathBuf::from(&temp_path), path)?;

    Ok(())
}

fn has_header(header: &[u8], path: &PathBuf) -> bool {
    let mut file = File::open(path).expect("Should be able to open header file.");
    let mut buf = vec![0 as u8; header.len()];

    file.read(&mut buf).expect("File should be readable.");

    let normalized_levenshtein_distance = normalized_levenshtein(
        from_utf8(&header).expect("Header should be UTF8."),
        from_utf8(&buf).expect("Beginning of file should be UTF8."),
    );
    if normalized_levenshtein_distance > 0.9 && normalized_levenshtein_distance < 1. {
        println!(
            "Header and beginning of file {:?} were {:?}% similar. Consider looking at it.",
            path.file_name().expect("File should have a name."),
            (normalized_levenshtein_distance * 10000.).round() / 100.,
        )
    };
    buf == header
}
