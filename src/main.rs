extern crate clap;
extern crate dividebatur;
extern crate rayon;
extern crate serde;
extern crate serde_derive;
extern crate toml;

use clap::{App, Arg};
use rayon::prelude::*;
use serde_derive::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::Read;
use dividebatur::engine::*;

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
    state: String,
    candidates: String,
    preferences: String,
    vacancies: usize,
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
            .map(|(slug, count)| {
                let dataset = match config.dataset.get(&count.dataset) {
                    Some(d) => d,
                    None => {
                        panic!();
                    }
                };
                CountTask {
                    state: slug.clone(),
                slug: slug.clone(),
                candidates: config.candidates.all.clone(),
                preferences: dataset.preferences.clone(),
                format: config.format.clone(),
                vacancies: count.vacancies,
            }})
            .collect();
        work.counts.append(&mut new);
    }
    work
}

fn run_task(task: &CountTask) -> Result<bool, String> {
    let candidates = match dividebatur::aec::data::candidates::load(&task.candidates, &task.state) {
        Ok(rows) => rows,
        Err(error) => {
            return Err(format!("Couldn't read candidates file: {:?}", error));
        }
    };
    let cd = dividebatur::senate2015::load_candidate_data(candidates);

    let prefpath = &task.preferences;
    let ballot_states =
        dividebatur::aec::data::formalpreferences::read_file(prefpath, &cd.tickets, cd.count);
    println!("len {}", cd.tickets.len());

    println!(
        "{} unique bundle states at commencement of count.",
        ballot_states.len()
    );

    let mut automation = VecDeque::new();
    automation.push_back(0);
    let mut engine = CountEngine::new(task.vacancies as u32, cd, ballot_states, automation);
    while {
        let outcome = engine.count();
        match outcome {
            CountOutcome::CountComplete(nrounds, state) => {
                // engine.print_debug();
                println!("{:?}", state);
                println!("Election complete after {} rounds of counting.", nrounds);
                false
            }
            CountOutcome::CountContinues(_, _) => {
                // engine.print_debug();
                true
            }
        }
    } {}
    Ok(true)
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

    let work = get_counts(matches.values_of("INPUT").unwrap().collect());
    let _x: Vec<_> = work
        .counts
        .par_iter()
        .map(|x| {
            let r = run_task(x);
            println!("{:?} -> {:?}", x, r);
        })
        .collect();
}
