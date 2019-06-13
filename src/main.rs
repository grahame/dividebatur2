extern crate clap;
extern crate dividebatur;
extern crate serde;
extern crate serde_derive;
extern crate toml;

use clap::{App, Arg};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize)]
struct Candidates {
    senate: String,
    all: String,
}

#[derive(Debug, Deserialize)]
struct Dataset {
    preferences: String,
}

#[derive(Debug, Deserialize)]
struct Tie {
    tie: Vec<u32>,
    pick: u32,
}

#[derive(Debug, Deserialize)]
struct Count {
    method: Option<String>,
    dataset: String,
    description: String,
    vacancies: usize,
    election_order_ties: Vec<Tie>,
    election_ties: Vec<Tie>,
    exclusion_ties: Vec<Tie>,
}

#[derive(Debug, Deserialize)]
struct Config {
    format: String,
    candidates: Candidates,
    dataset: HashMap<String, Dataset>,
    count: HashMap<String, Count>,
}

fn read_config(input_file: &str) -> Result<Config, String> {
    let mut fd = match File::open(input_file) {
        Ok(fd) => fd,
        Err(e) => return Err(format!("unable to read {}: {}", input_file, e)),
    };

    let mut buf = String::new();
    fd.read_to_string(&mut buf).unwrap();

    let config: Config = match toml::from_str(&buf) {
        Ok(c) => c,
        Err(e) => return Err(format!("unable to parse {}: {}", input_file, e)),
    };

    println!("{:?}", config);

    Ok(config)
}

#[derive(Debug)]
struct CountTask {
    format: String,
    slug: String,
}

#[derive(Debug)]
struct Work {
    counts: Vec<CountTask>,
}

fn get_counts(input_files: Vec<&str>) -> Work {
    let mut work = Work { counts: Vec::new() };
    for fname in input_files {
        println!("-> {}", fname);
        let config = match read_config(fname) {
            Ok(c) => c,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        let mut new = config
            .count
            .iter()
            .map(|(slug, x)| CountTask {
                slug: slug.clone(),
                format: config.format.clone(),
            })
            .collect();
        work.counts.append(&mut new);
    }
    work
}

fn main() {
    let matches = App::new("dividebatur")
        .version("0.4.0")
        .about("single transferable vote counter")
        .author("Grahame Bowland <grahame@oreamnos.com.au>")
        .arg(
            Arg::with_name("INPUT")
                .multiple(true)
                .required(true)
                .help("input file (TOML)"),
        )
        .get_matches();

    let config = get_counts(matches.values_of("INPUT").unwrap().collect());
    println!("config: {:?}", config);
}
