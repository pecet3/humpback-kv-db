use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn parse_lines_from_file(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let lines: Vec<String> = reader
        .lines()
        .filter_map(Result::ok) // tylko poprawne linie
        .collect();

    Ok(lines)
}
