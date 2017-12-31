use std::collections::HashMap;
use defs::*;
use num::FromPrimitive;
use num::rational::{Ratio};

#[derive(Debug)]
pub enum CountOutcome {
    CountComplete(u32),
    CountContinues(u32)
}

#[derive(Debug)]
pub struct CountEngine {
    candidates: u32,
    vacancies: u32,
    candidate_bundle_transactions: CandidateToBundleTransaction,
    total_papers: u32,
    pub counts: u32,
    quota: u32,
    elected: Vec<CandidateIndex>,
    excluded: Vec<CandidateIndex>
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


impl CountEngine {
    fn determine_quota(total_papers: u32, vacancies: u32) -> u32 {
        (total_papers / (vacancies + 1)) + 1
    }

    pub fn new(vacancies: u32, candidates: u32, ballot_states: Vec<BallotState>) -> CountEngine {
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
        CountEngine {
            candidates: candidates,
            vacancies: vacancies,
            candidate_bundle_transactions: ctbt,
            total_papers: total_papers,
            counts: 0,
            quota: CountEngine::determine_quota(total_papers, vacancies),
            elected: Vec::new(),
            excluded: Vec::new(),
        }
    }

    pub fn print_debug(&self, cd: &CandidateData) {
        println!("-- CountEngine::print_debug (round {}) --", self.counts);
        println!("Candidates: {}", self.candidates);
        println!("Total papers: {}", self.total_papers);
        println!("Quota: {}", self.quota);
        println!("Candidate totals:");
        let mut cbt: Vec<(&CandidateIndex, &CandidateBundleTransactions)> = self.candidate_bundle_transactions.iter().collect();
        cbt.sort_by(|a, b| a.0.cmp(b.0));
        for (candidate_id, cbt) in cbt {
            let a: u32 = cbt.total_votes();
            println!("    {:?} {} votes for candidate {} ({})", candidate_id, a, cd.get_name(*candidate_id), cd.get_party(*candidate_id));
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

    fn elect(&mut self, candidate: CandidateIndex) {
        if self.elected.contains(&candidate) { 
            panic!("Candidate elected twice");
        }
        println!("Elected candidate: {:?}", candidate);
        self.elected.push(candidate);
    }

    pub fn count(&mut self) -> CountOutcome {
        // count votes, once (a single 'round')
        self.counts += 1;
        // action execution to come

        // has anyone been elected in this count?
        let newly_elected = self.determine_elected_candidates();
        for candidate in newly_elected {
            self.elect(candidate);
            if self.elected.len() as u32 == self.vacancies {
                return CountOutcome::CountComplete(self.counts);
            }
        }
        panic!("unreachable");
        return CountOutcome::CountContinues(self.counts);
    }
}
