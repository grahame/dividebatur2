extern crate clap;
extern crate dividebatur;
extern crate rayon;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use clap::{App, Arg};
use dividebatur::engine::*;
use rayon::prelude::*;
use std::collections::{VecDeque};
use std::fs::File;
use dividebatur::configuration::{read_config, CountTask};

fn run_task(task: &CountTask) -> Result<bool, String> {
    println!("-> running task: {:?}", task);
    let candidates = match dividebatur::aec::data::candidates::load(&task.candidates, &task.state) {
        Ok(rows) => rows,
        Err(error) => {
            return Err(format!("Couldn't read candidates file: {:?}", error));
        }
    };
    let cd = dividebatur::senate2015::load_candidate_data(candidates);

    let output_file = format!("output/{}.json", task.slug);
    let _fd = File::create(output_file);
    // serde_json::to_writer(fd, cd);

    let prefpath = &task.preferences;
    let ballot_states =
        dividebatur::aec::data::formalpreferences::read_file(prefpath, &cd.tickets, cd.count);

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

    let work = read_config(matches.values_of("INPUT").unwrap().collect());
    let _x: Vec<_> = work
        .counts
        .par_iter()
        .map(|x| {
            let r = run_task(x);
            println!("{:?} -> {:?}", x, r);
        })
        .collect();
}
