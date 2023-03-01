use rayon::prelude::*;
use std::{
    fs::{read_dir, remove_file, rename, File},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_through_dir("/// Oi galera!\n\n", "testing")?;
    // prepend_to_file("/// Oi galera!\n\n", "testing/test.txt")?;

    Ok(())
}

fn run_through_dir(header: &str, dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dir = read_dir(&dir_path)?;

    dir.par_bridge().for_each(|file| {
        let file = file.expect("Should be able to access files in folder.");
        prepend_to_file(header, &file.path()).expect("Should be able to prepend to file.");
    });

    Ok(())
}

fn prepend_to_file(header: &str, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;

    let mut reader = BufReader::new(file);

    let parent = path.parent().unwrap();
    let temp_path = parent.join(path.file_name().unwrap().to_str().unwrap().to_owned() + "_temp_");
    let temp = File::create(&temp_path)?;

    let mut temp_writer = BufWriter::new(temp);

    temp_writer.write_all(header.as_bytes())?;

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
