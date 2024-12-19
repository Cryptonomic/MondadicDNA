use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use log::info;

use std::path::Path;
use reqwest::blocking::get;

pub fn process_file(filename: &str, num_lines: usize) -> io::Result<HashMap<u64, u8>> {
    let file_path: String;

    // Check if filename is a URL
    if filename.starts_with("http://") || filename.starts_with("https://") {
        println!("Downloading file from URL: {}", filename);
        file_path = download_file(filename, "genome_data_file.txt")?;
    } else {
        file_path = filename.to_string();
    }



    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut results = HashMap::new();

    for line in reader.lines().filter_map(|line| line.ok()) {
        if !line.starts_with('#') {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 4 {
                let rsid = parts[0].to_string();
                let genotype = parts[3];

                let encoded_rsid = encode_rsid(&rsid);
                let encoded_genotype = encode_genotype(genotype);

                // Insert into HashMap
                results.insert(encoded_rsid, encoded_genotype);
                if results.len() == num_lines {
                    info!("Reached the limit of lines to process: {:?}", num_lines);
                    break;
                }
            }
        }
    }

    Ok(results)
}

pub fn encode_rsid(rsid: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    rsid.trim_start_matches("rs").hash(&mut hasher);
    return hasher.finish();
}

pub fn get_genotype_encoding_map() -> HashMap<&'static str, u8> {
    HashMap::from([
        ("AA", 1),
        ("AC", 2), ("CA", 2),
        ("AG", 3), ("GA", 3),
        ("AT", 4), ("TA", 4),
        ("CC", 5),
        ("CG", 6), ("GC", 6),
        ("CT", 7), ("TC", 7),
        ("GG", 8),
        ("GT", 9), ("TG", 9),
        ("TT", 10),
        ("NN", 11),
        ("", 12), ("???", 12),
    ])
}

pub fn encode_genotype(genotype: &str) -> u8 {
    let encoding_map = get_genotype_encoding_map();
    *encoding_map.get(genotype).unwrap_or(&0)
}

fn download_file(url: &str, output_path: &str) -> io::Result<String> {
    let response = get(url).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut file = File::create(output_path)?;
    std::io::copy(&mut response.bytes().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?.as_ref(), &mut file)?;

    Ok(output_path.to_string())
}
