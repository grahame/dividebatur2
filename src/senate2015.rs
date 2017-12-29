use std::collections::HashMap;
use defs::*;
use aec;
use num::FromPrimitive;
use num::rational::{Ratio};

fn load_candidate_data(candidates: Vec<aec::data::candidates::AECAllCandidateRow>) -> CandidateData {
    let mut names = Vec::new();
    let mut parties = Vec::new();

    let mut current_ticket = String::from("");
    let mut tickets = Vec::new();

    // NB: the Candidate Rows are sorted into ballot paper order
    for (idx, candidate) in candidates.iter().enumerate() {
        names.push(format!("{}, {}", candidate.surname, candidate.ballot_given_nm));
        parties.push(candidate.party_ballot_nm.clone());
        if candidate.ticket == "UG" {
            continue;
        }

        if candidate.ticket != current_ticket {
            tickets.push(Vec::new());
            current_ticket = candidate.ticket.clone();
        }

        let p = tickets.len() - 1;
        tickets[p].push(CandidateIndex(idx as u8));
    }
    CandidateData {
        count: candidates.len(),
        names: names,
        parties: parties,
        tickets: tickets
    }
}

// all bundle transactions held by a candidate, in a given round of the count
#[derive(Debug)]
struct CandidateBundleTransactions {
    bundle_transactions: Vec<BundleTransaction>
}

type CandidateToBundleTransaction = HashMap<CandidateIndex, CandidateBundleTransactions>;

impl CandidateBundleTransactions {
    fn total_votes(&self) -> u32 {
        self.bundle_transactions.iter().map(|bt| bt.votes).sum()
    }
    fn new() -> CandidateBundleTransactions {
        CandidateBundleTransactions {
            bundle_transactions: Vec::new()
        }
    }
}

#[derive(Debug)]
struct SenateCount {
    candidates: u32,
    vacancies: u32,
    candidate_bundle_transactions: CandidateToBundleTransaction,
    total_papers: u32,
    counts: u32,
    quota: u32,
    elected: Vec<CandidateIndex>,
    excluded: Vec<CandidateIndex>
}

impl SenateCount {
    fn determine_quota(total_papers: u32, vacancies: u32) -> u32 {
        (total_papers / (vacancies + 1)) + 1
    }

    fn new(vacancies: u32, candidates: u32, ballot_states: Vec<BallotState>) -> SenateCount {
        let mut by_candidate: HashMap<CandidateIndex, Vec<BallotState>> = HashMap::new();
        for ballot_state in ballot_states.into_iter() {
            let candidate_id = match ballot_state.current_preference() {
                Some(p) => p,
                None => panic!("informal ballot in initial ballots")
            };
            let v = by_candidate.entry(candidate_id).or_insert(Vec::new());
            v.push(ballot_state);
        }
        let mut ctbt = HashMap::new();
        let ratio_one = Ratio::from_integer(FromPrimitive::from_u32(1).unwrap()) / Ratio::from_integer(FromPrimitive::from_u32(1).unwrap());
        let mut total_papers = 0;
        for (candidate_id, ballot_states) in by_candidate.drain() {
            let t = ctbt.entry(candidate_id).or_insert(CandidateBundleTransactions::new());
            let votes = ballot_states.iter().map(|bs| bs.count).sum();
            total_papers += votes;
            let bt = BundleTransaction {
                ballot_states: ballot_states,
                transfer_value: ratio_one.clone(),
                votes: votes,
            };
            t.bundle_transactions.push(bt);
        }
        SenateCount {
            candidates: candidates,
            vacancies: vacancies,
            candidate_bundle_transactions: ctbt,
            total_papers: total_papers,
            counts: 0,
            quota: SenateCount::determine_quota(total_papers, vacancies),
            elected: Vec::new(),
            excluded: Vec::new(),
        }
    }

    fn print_debug(&self, cd: &CandidateData) {
        println!("-- SenateCount::print_debug (round {}) --", self.counts);
        println!("Candidates: {}", self.candidates);
        println!("Total papers: {}", self.total_papers);
        println!("Quota: {}", self.quota);
        println!("Candidate totals:");
        for (candidate_id, cbt) in self.candidate_bundle_transactions.iter() {
            let a: u32 = cbt.total_votes();
            println!("    {} votes for candidate {} ({})", a, cd.get_name(*candidate_id), cd.get_party(*candidate_id));
        }
        println!("Candidates elected: {:?}", self.elected);
        println!("Candidates excluded: {:?}", self.excluded);
    }

    fn determine_elected_candidates(&mut self) -> Vec<CandidateIndex> {
        // determine all candidates whose vote total is over the threshold; bin by
        // the number of votes they are holding, so we can determine any ties
        let mut votes_candidate: HashMap<u32, Vec<CandidateIndex>> = HashMap::new();
        for (candidate_id, cbt) in self.candidate_bundle_transactions.iter() {
            let votes = cbt.total_votes();
            if votes > self.quota {
                let v = votes_candidate.entry(cbt.total_votes()).or_insert(Vec::new());
                v.push(*candidate_id);
            }
        }
 
        let mut elected: Vec<CandidateIndex> = Vec::new();
        let mut possible: Vec<(&u32, &Vec<CandidateIndex>)> = votes_candidate.iter().collect();
        possible.sort_by(|a, b| b.0.cmp(a.0));
        for (_votes, candidate_ids) in possible.into_iter() {
            // no tie in the ordering: elect this candidate
            if candidate_ids.len() == 1 {
                elected.push(candidate_ids[0]);
            } else {
                panic!("Election ordering ties are not yet implemented.");
            }
        }
        elected
    }

    fn execute_count(&mut self) -> bool {
        self.counts += 1;
        // action execution to come

        // has anyone been elected in this count?
        let newly_elected = self.determine_elected_candidates();
        for candidate in newly_elected {
            self.elected.push(candidate);
            if self.elected.len() as u32 == self.vacancies {
                return false;
            }
        }
        panic!("unreachable");
    }
}

pub fn run() {
    let candidates = match aec::data::candidates::load("aec_data/fed2016/common/aec-senate-candidateinformation-20499.csv", "NT") {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read candidates file: {:?}", error);
        }
    };
    let cd = load_candidate_data(candidates);

    let ballot_states = match aec::data::formalpreferences::load("aec_data/fed2016/nt/data/aec-senate-formalpreferences-20499-NT.csv", &cd) {
        Ok(data) => data,
        Err(error) => {
            panic!("Couldn't read formal preferences file: {:?}", error);
        }
    };

    println!("{} unique bundle states at commencement of count.", ballot_states.len());

    let candidate_count = cd.count.clone();
    let mut count = SenateCount::new(2, candidate_count as u32, ballot_states);
    while {
        let done = count.execute_count();
        count.print_debug(&cd);
        done
    } { }
    println!("Count terminated after {} counts.", count.counts);
}
