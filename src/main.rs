#[macro_use]
extern crate serde_derive;
extern crate rayon;
mod aec;
use std::collections::HashMap;

fn load_groups(candidates: Vec<aec::data::candidates::AECAllCandidateRow>) {
    let mut tickets = Vec::new();
    let mut ticket_candidates = HashMap::new();

    for (idx, candidate) in candidates.iter().enumerate() {
        ticket_candidates.entry(candidate.ticket.clone()).or_insert_with(|| {
            tickets.push(candidate.ticket.clone());
            Vec::new()
        }).push(idx);
    }
    println!("{:?}", tickets);
    println!("{:?}", ticket_candidates);
}

fn senate2015() {
    let candidates = match aec::data::candidates::load("aec_data/fed2016/common/aec-senate-candidateinformation-20499.csv", "NT") {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read candidates file: {:?}", error);
        }
    };

    let groups = load_groups(candidates);

    return;

    println!("{} candidates", candidates.len());

    let prefs = match aec::data::formalpreferences::load("aec_data/fed2016/wa/data/aec-senate-formalpreferences-20499-WA.csv") {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read formal preferences file: {:?}", error);
        }
    };

    println!("{} formal preferences", prefs.len());

}

fn main() {
    senate2015();
}
