extern crate clap;
extern crate dividebatur;
extern crate serde;
extern crate serde_derive;
extern crate toml;

use clap::{App, Arg};
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct Candidates {
    senate: String,
    all: String
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

fn run(input_file: &str) {
    println!("-> {}", input_file);
    let config = match read_config(input_file) {
        Ok(c) => c,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
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

    for input_file in matches.values_of("INPUT").unwrap() {
        run(input_file);
    }
}
