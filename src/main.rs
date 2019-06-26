extern crate clap;
extern crate dividebatur;
extern crate rayon;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use clap::{App, Arg};
use dividebatur::configuration::{read_config, CountGroup, CountTask};
use dividebatur::engine::*;
use dividebatur::output::{write_summary, CountOutput};
use rayon::prelude::*;
use std::collections::VecDeque;

fn run_task(group: &CountGroup, task: &CountTask, debug: bool) -> Result<bool, String> {
    if debug {
        println!("-> running task: {}", task.description);
    }
    let mut output: CountOutput = CountOutput::new(&task.slug);
    let candidates = match dividebatur::aec::data::candidates::load(&task.candidates, &task.state) {
        Ok(rows) => rows,
        Err(error) => {
            return Err(format!("Couldn't read candidates file: {:?}", error));
        }
    };
    let cd = dividebatur::senate2015::load_candidate_data(candidates);
    output.set_candidates(&cd);

    let prefpath = &task.preferences;
    let ballot_states =
        dividebatur::aec::data::formalpreferences::read_file(prefpath, &cd.tickets, cd.count);

    if debug {
        println!(
            "{} unique bundle states at commencement of count.",
            ballot_states.len()
        );
    }

    let mut automation = VecDeque::new();
    automation.push_back(0);
    let mut engine = CountEngine::new(task.vacancies as u32, cd, ballot_states, automation);
    output.set_parameters(&group, &task, &engine);
    while {
        let outcome = engine.count();
        match outcome {
            CountOutcome::CountComplete(nrounds, _state) => {
                if debug {
                    engine.print_debug();
                    println!("Election complete after {} rounds of counting.", nrounds);
                }
                false
            }
            CountOutcome::CountContinues(_, _) => {
                if debug {
                    engine.print_debug();
                }
                true
            }
        }
    } {}
    output.close();
    Ok(true)
}

fn main() {
    let matches = App::new("dividebatur")
        .version("0.4.0")
        .about("single transferable vote counter")
        .author("Grahame Bowland <grahame@oreamnos.com.au>")
        .arg(
            Arg::with_name("debug")
                .short("d")
                .help("Enable debugging output"),
        )
        .arg(
            Arg::with_name("INPUT")
                .multiple(true)
                .required(true)
                .help("input file (TOML)"),
        )
        .get_matches();

    let debug = matches.occurrences_of("debug") > 0;
    let work = read_config(matches.values_of("INPUT").unwrap().collect());
    write_summary(&work);
    for group in work.groups {
        let _x: Vec<_> = group
            .counts
            .par_iter()
            .map(|task| {
                let _result = run_task(&group, task, debug);
            })
            .collect();
    }
}
