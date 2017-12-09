use std::collections::HashMap;
use defs::*;
use aec;

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

pub fn run() {
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

    println!("{} unique preferences read.", prefs.len());
}
