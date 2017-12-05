#[macro_use]
extern crate serde_derive;
extern crate rayon;
mod aec;
use std::collections::HashMap;

// represents a candidate's index on the ballot paper
// ranges from 0..N-1 where N is the number of candidates
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct CandidateIndex(u8);

// represents a group's index on the ballot paper
// ranges from 0..N-1 where N is the number of groups
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct GroupIndex(u8);

// a voter's numerical preference for a candidate
// if valid, it ranges from 1..N where N is the number of candidates
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PreferenceForCandidate(u8);

// a voter's numerical preference for a group
// if valid, it ranges from 1..N where N is the number of groups
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PreferenceForGroup(u8);

#[derive(Debug)]
pub struct CandidateData {
    count: usize,
    tickets: Vec<String>,
    ticket_candidates: HashMap<String, Vec<CandidateIndex>>
}

fn load_groups(candidates: Vec<aec::data::candidates::AECAllCandidateRow>) -> CandidateData {
    let mut tickets = Vec::new();
    let mut ticket_candidates: HashMap<String, Vec<CandidateIndex>> = HashMap::new();

    for (idx, candidate) in candidates.iter().enumerate() {
        if candidate.ticket == "UG" {
            continue;
        }
        ticket_candidates.entry(candidate.ticket.clone()).or_insert_with(|| {
            tickets.push(candidate.ticket.clone());
            Vec::new()
        }).push(CandidateIndex(idx as u8));
    }
    CandidateData {
        count: candidates.len(),
        tickets: tickets,
        ticket_candidates: ticket_candidates
    }
}

fn senate2015() {
    let candidates = match aec::data::candidates::load("aec_data/fed2016/common/aec-senate-candidateinformation-20499.csv", "NSW") {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read candidates file: {:?}", error);
        }
    };
    let cd = load_groups(candidates);

    let prefs = match aec::data::formalpreferences::load("aec_data/fed2016/nsw/data/aec-senate-formalpreferences-20499-NSW.csv", &cd) {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read formal preferences file: {:?}", error);
        }
    };

    println!("{} formal preferences read.", prefs.len());
}

fn main() {
    senate2015();
}
