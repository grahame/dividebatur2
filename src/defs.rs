/*
 * core types
 */

use num::rational::BigRational;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
/// candidate's index on the ballot paper
/// ranges from `0..N-1` where `N` is the number of candidates
pub struct CandidateIndex(pub u8);

/// group's index on the ballot paper
/// ranges from `0..N-1` where N is the number of groups
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct GroupIndex(pub u8);

#[derive(Debug)]
/// `count` ballots in the count, all with the same `form`,
/// expressing `active_preference`
pub struct BallotState {
    pub form: Vec<CandidateIndex>,
    pub count: u32,
    pub active_preference: usize,
}

#[derive(Default)]
pub struct CountResults {
    elected: Vec<CandidateIndex>,
    excluded: Vec<CandidateIndex>,
    inactive: HashSet<CandidateIndex>, // either elected or excluded; fast lookup
}

impl CountResults {
    pub fn new() -> CountResults {
        CountResults {
            elected: Vec::new(),
            excluded: Vec::new(),
            inactive: HashSet::new(),
        }
    }

    pub fn number_elected(&self) -> u32 {
        self.elected.len() as u32
    }

    pub fn get_elected(&self) -> &Vec<CandidateIndex> {
        &self.elected
    }

    pub fn get_excluded(&self) -> &Vec<CandidateIndex> {
        &self.excluded
    }

    pub fn candidate_elected(&mut self, candidate: CandidateIndex) {
        self.elected.push(candidate);
        self.inactive.insert(candidate);
    }

    pub fn candidate_excluded(&mut self, candidate: CandidateIndex) {
        self.excluded.push(candidate);
        self.inactive.insert(candidate);
    }

    pub fn candidate_is_inactive(&self, candidate: CandidateIndex) -> bool {
        self.inactive.contains(&candidate)
    }
}

impl BallotState {
    pub fn alive(&self) -> bool {
        self.active_preference < self.form.len()
    }

    pub fn current_preference(&self) -> Option<CandidateIndex> {
        if self.alive() {
            Some(self.form[self.active_preference])
        } else {
            None
        }
    }

    pub fn goto_next_preference(&mut self, results: &CountResults) {
        loop {
            self.active_preference += 1;
            match self.current_preference() {
                Some(candidate) => {
                    if !results.candidate_is_inactive(candidate) {
                        break;
                    }
                }
                None => {
                    break;
                }
            }
        }
    }
}

/// a collection of ballot states, all of which were transferred to
/// the total of a candidate during a count. the member `votes`
/// represents the integer value of the votes transferred to the
/// candidate, after the application of the transfer value to the
/// total number of papers in the transaction
#[derive(Debug)]
pub struct BundleTransaction {
    pub ballot_states: Vec<BallotState>,
    pub transfer_value: BigRational,
    pub votes: u32,
    pub papers: u32,
}

#[derive(Debug)]
pub struct CandidateData {
    pub count: usize,
    pub names: Vec<String>,
    pub parties: Vec<String>,
    pub tickets: Vec<Vec<CandidateIndex>>,
}

impl CandidateData {
    pub fn vec_names(&self, candidates: &[CandidateIndex]) -> String {
        let names: Vec<String> = candidates.iter().map(|c| self.get_name(*c)).collect();
        names.join("; ")
    }
}

impl CandidateData {
    pub fn get_name(&self, idx: CandidateIndex) -> String {
        self.names[idx.0 as usize].clone()
    }
    pub fn get_party(&self, idx: CandidateIndex) -> String {
        self.parties[idx.0 as usize].clone()
    }
}
