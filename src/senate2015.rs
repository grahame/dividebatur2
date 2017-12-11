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

// all bundle transactions held by a candidate, in a given round of the count
type CandidateBundleTransaction = Vec<BundleTransaction>;
type CandidateToBundleTransaction = HashMap<CandidateIndex, CandidateBundleTransaction>;

#[derive(Debug)]
struct CountState {
    candidate_bundle_transactions: CandidateToBundleTransaction
}

fn build_initial_state(ballot_states: Vec<BallotState>) -> CountState {
    let mut by_candidate: HashMap<CandidateIndex, Vec<BallotState>> = HashMap::new();
    for ballot_state in ballot_states.into_iter() {
        let pref = match ballot_state.current_preference() {
            Some(p) => p,
            None => panic!("informal ballot in initial ballots")
        };
        let v = by_candidate.entry(pref).or_insert(Vec::new());
        v.push(ballot_state);
    }
    let mut ctbt = HashMap::new();
    for (candidate_id, ballot_states) in by_candidate.drain() {
        let t = ctbt.entry(candidate_id).or_insert(CandidateBundleTransaction::new());
        let votes = ballot_states.iter().map(|bs| bs.count).sum();
        let bt = BundleTransaction {
            ballot_states: ballot_states,
            transfer_value: 1,
            votes: votes,
        };
        t.push(bt);
    }
    CountState {
        candidate_bundle_transactions: ctbt
    }
}

pub fn run() {
    let candidates = match aec::data::candidates::load("aec_data/fed2016/common/aec-senate-candidateinformation-20499.csv", "NT") {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read candidates file: {:?}", error);
        }
    };
    let cd = load_groups(candidates);

    let ballot_states = match aec::data::formalpreferences::load("aec_data/fed2016/nt/data/aec-senate-formalpreferences-20499-NT.csv", &cd) {
        Ok(data) => data,
        Err(error) => {
            panic!("Couldn't read formal preferences file: {:?}", error);
        }
    };

    println!("{} unique bundle states at commencement of count.", ballot_states.len());

    let state = build_initial_state(ballot_states);
    let mut total = 0;
    for (candidate_id, cbt) in state.candidate_bundle_transactions {
        let a: u32 = cbt.iter().map(|bt| bt.votes).sum();
        println!("{} votes for candidate_id {:?}", a, candidate_id);
        total += a;
    }
    println!("total = {}", total);
}
