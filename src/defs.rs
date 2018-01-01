/*
 * core types
 */

use num::rational::{BigRational};
use std::collections::{HashSet};

// represents a candidate's index on the ballot paper
// ranges from 0..N-1 where N is the number of candidates
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct CandidateIndex(pub u8);

// represents a group's index on the ballot paper
// ranges from 0..N-1 where N is the number of groups
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct GroupIndex(pub u8);

// one ore more ballots entered into the count, all with the
// same form, and the current state of those ballots within
// the count (e.g. the current preference)
#[derive(Debug)]
pub struct BallotState { 
    pub form: Vec<CandidateIndex>,
    pub count: u32,
    pub active_preference: usize,
}

impl BallotState {
    fn alive(&self) -> bool {
        self.active_preference < self.form.len()
    }

    pub fn current_preference(&self) -> Option<CandidateIndex> {
        if self.alive() {
            Some(self.form[self.active_preference])
        } else {
            None
        }
    }

    pub fn to_next_preference(&mut self) -> Option<CandidateIndex> {
        self.active_preference += 1;
        return self.current_preference();
    }
}

// a collection of ballot states, all of which were transferred to
// the total of a candidate during a count. the member `votes`
// represents the integer value of the votes transferred to the
// candidate, after the application of the transfer value to the
// total number of papers in the transaction
pub struct BundleTransaction {
    pub ballot_states: Vec<BallotState>,
    pub transfer_value: BigRational,
    pub votes: u32,
    pub papers: u32,
}

pub struct CandidateData {
    pub count: usize,
    pub names: Vec<String>,
    pub parties: Vec<String>,
    pub tickets: Vec<Vec<CandidateIndex>>
}

impl CandidateData {
    pub fn vec_names(&self, candidates: &Vec<CandidateIndex>) -> String {
        let names: Vec<String> = candidates.iter().map(|c| self.get_name(*c)).collect();
        names.join(", ")
    }
    pub fn hashset_names(&self, candidate_set: &HashSet<CandidateIndex>) -> String {
        let mut candidates: Vec<CandidateIndex> = Vec::new();
        for candidate in candidate_set {
            candidates.push(candidate.clone());
        }
        self.vec_names(&candidates)
    }
}

impl CandidateData {
    pub fn get_name(&self, idx: CandidateIndex) -> String {
        return self.names[idx.0 as usize].clone();
    }
    pub fn get_party(&self, idx: CandidateIndex) -> String {
        return self.parties[idx.0 as usize].clone();
    }
}
