use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
use std::{
    fs::{read, read_dir, remove_file, rename, File},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
    str::from_utf8,
};
use strsim::normalized_levenshtein;

const BUFFER_MAX_BYTE_SIZE: usize = 4000;

/// Idempotent header prepender.
#[derive(Parser, Debug)]
struct Cli {
    /// File containing the header.
    header_file: String,

    /// Target folder containing files to add header.
    target_folder: String,

    /// Regex to match file names that will be considered for the headers.
    #[arg(short, long)]
    matching: Option<String>,

    /// Recursively runs through the target directory, visiting inner directories.
    #[arg(short, long, default_value_t = false)]
    recursive: bool,

    /// Flag for deleting the header if it exists rather than prepending.
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
    let target_folder = PathBuf::from(target_folder);

    let reg = if let Some(x) = &args.matching {
        Regex::new(&x).expect("Matching should be valid regex.")
    } else {
        Regex::new(r".*").expect("'.*' should be valid regex.")
    };

    if args.recursive {
        visit_dirs(&target_folder, &reg, args.delete, &header)?;
    } else {
        run_through_dir(&header, &target_folder, reg, args.delete)?;
    }
    Ok(())
}

fn run_through_dir(
    header: &[u8],
    dir_path: &PathBuf,
    reg: Regex,
    should_delete: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = read_dir(&dir_path)?;

    dir.par_bridge().for_each(|file| {
        let file = file.expect("Files in directory should be interactable.");

        if file
            .metadata()
            .expect("Should be able to get file metadata.")
            .is_dir()
        {
            return ();
        }

        process_file(file, &reg, should_delete, header);
    });

    Ok(())
}

fn process_file(file: std::fs::DirEntry, reg: &regex::Regex, should_delete: bool, header: &[u8]) {
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
}

fn prepend_to_file(header: &[u8], path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if has_header(header, path) {
        return Ok(());
    }

    let file = File::open(path)?;

    let mut reader = BufReader::new(file);

    let parent = path.parent().unwrap();
    let temp_file = mktemp::Temp::new_file_in(parent)?;
    let temp_path = temp_file.as_path();
    let temp_file = File::create(&temp_file)?;

    let mut temp_writer = BufWriter::new(temp_file);

    temp_writer.write_all(header)?;

    let mut buf = [0; BUFFER_MAX_BYTE_SIZE];

    loop {
        let n = reader.read(&mut buf)?;
        temp_writer.write_all(&buf[0..n])?;

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
    let temp_file = mktemp::Temp::new_file_in(parent)?;
    let temp_path = temp_file.as_path();
    let temp_file = File::create(&temp_file)?;

    let mut temp_writer = BufWriter::new(temp_file);

    let mut buf_to_throw_away = vec![0; header.len()];

    reader.read(&mut buf_to_throw_away)?;

    let mut buf = [0; BUFFER_MAX_BYTE_SIZE];

    loop {
        let n = reader.read(&mut buf)?;
        temp_writer.write_all(&buf[0..n])?;

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

fn visit_dirs(
    dir: &PathBuf,
    regex: &Regex,
    should_delete: bool,
    header: &[u8],
) -> Result<(), String> {
    if dir.is_dir() {
        read_dir(dir)
            .expect("Should be able to read directory.")
            .par_bridge()
            .for_each(|file| {
                if let Ok(x) = file {
                    let path = x.path();
                    if path.is_dir() {
                        visit_dirs(&path, regex, should_delete, header)
                            .expect("Unexpected non-directory inside visit_dirs loop");
                    } else {
                        process_file(x, regex, should_delete, header);
                    }
                }
            });

        Ok(())
    } else {
        Err("Not a directory.".to_owned())
    }
}
