use std::{
    fs::{remove_file, rename, File},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    prepend_to_file("/// Oi galera!\n\n", "testing/test.txt")?;

    Ok(())
}

fn prepend_to_file(header: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(PathBuf::from(&path))?;
    let mut reader = BufReader::new(file);

    let temp_path = path.to_owned() + "_temp_";
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
