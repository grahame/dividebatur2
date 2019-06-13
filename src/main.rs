extern crate clap;
extern crate dividebatur;
extern crate num_format;
extern crate rayon;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use clap::{App, Arg};
use dividebatur::configuration::{read_config, CountGroup, CountTask};
use dividebatur::engine::*;
use dividebatur::output::{write_summary, CountOutput};
use num_format::{Locale, ToFormattedString};
use rayon::prelude::*;
use std::collections::VecDeque;
use std::time::Instant;

struct TaskSummary {
    total_papers: u32,
}

fn run_task(group: &CountGroup, task: &CountTask, debug: bool) -> Result<TaskSummary, String> {
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
            CountOutcome::CountComplete(ncounts, _state) => {
                if debug {
                    engine.print_debug();
                    println!("Election complete after {} counts.", ncounts);
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
    Ok(TaskSummary {
        total_papers: engine.total_papers,
    })
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
    let mut elapsed: Vec<(String, String, u32, u128)> = Vec::new();
    for group in work.groups {
        let mut stats: Vec<(String, String, u32, u128)> = group
            .counts
            .par_iter()
            .map(|task| {
                let start = Instant::now();
                let result = run_task(&group, task, debug).unwrap();
                (
                    group.filename.clone(),
                    task.slug.clone(),
                    result.total_papers,
                    start.elapsed().as_millis(),
                )
            })
            .collect();
        elapsed.append(&mut stats);
    }

    elapsed.sort_by_key(|(_filename, _slug, _papers, time_ms)| time_ms.clone());
    elapsed.reverse();
    println!("|----------------------|--------------|--------------|--------------|");
    println!("| config               | count        | papers       | elapsed (ms) |");
    println!("|----------------------|--------------|--------------|--------------|");
    for (filename, slug, papers, time_ms) in elapsed {
        println!(
            "| {:<20} | {:<12} | {:>12} | {:>12} |",
            filename,
            slug,
            papers.to_formatted_string(&Locale::en),
            time_ms.to_formatted_string(&Locale::en)
        );
    }
    println!("|----------------------|--------------|--------------|--------------|");
}
