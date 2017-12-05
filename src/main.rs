#[macro_use]
extern crate serde_derive;
extern crate rayon;
mod aec;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CandidateData {
    count: u8,
    tickets: Vec<String>,
    ticket_candidates: HashMap<String, Vec<u8>>
}

fn load_groups(candidates: Vec<aec::data::candidates::AECAllCandidateRow>) -> CandidateData {
    let mut tickets = Vec::new();
    let mut ticket_candidates = HashMap::new();

    for (idx, candidate) in candidates.iter().enumerate() {
        ticket_candidates.entry(candidate.ticket.clone()).or_insert_with(|| {
            tickets.push(candidate.ticket.clone());
            Vec::new()
        }).push(idx as u8);
    }
    CandidateData {
        count: candidates.len() as u8,
        tickets: tickets,
        ticket_candidates: ticket_candidates
    }
}

fn senate2015() {
    let candidates = match aec::data::candidates::load("aec_data/fed2016/common/aec-senate-candidateinformation-20499.csv", "NT") {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read candidates file: {:?}", error);
        }
    };
    println!("{} candidates", candidates.len());
    let cd = load_groups(candidates);

    let prefs = match aec::data::formalpreferences::load("aec_data/fed2016/nt/data/aec-senate-formalpreferences-20499-NT.csv", &cd) {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read formal preferences file: {:?}", error);
        }
    };

    println!("{:?}", cd);


    println!("{} formal preferences", prefs.len());

}

fn main() {
    senate2015();
}
