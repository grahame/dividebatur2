/*
 * core types
 */

use std::collections::HashMap;

// represents a candidate's index on the ballot paper
// ranges from 0..N-1 where N is the number of candidates
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct CandidateIndex(pub u8);

// represents a group's index on the ballot paper
// ranges from 0..N-1 where N is the number of groups
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct GroupIndex(pub u8);

// a voter's numerical preference for a candidate
// if valid, it ranges from 1..N where N is the number of candidates
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PreferenceForCandidate(pub u8);

// a voter's numerical preference for a group
// if valid, it ranges from 1..N where N is the number of groups
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PreferenceForGroup(pub u8);

// one ore more ballots entered into the count, all with the
// same form, and the current state of those ballots within
// the count (e.g. the current preference)
#[derive(Debug)]
pub struct BallotState { 
    pub form: Vec<CandidateIndex>,
    pub count: u32,
    pub active_preference: usize
}

impl BallotState {
    pub fn current_preference(&self) -> Option<CandidateIndex> {
        if self.active_preference < self.form.len() {
            Some(self.form[self.active_preference])
        } else {
            None
        }
    }
}

// a collection of ballot states, all of which were transferred to
// the total of a candidate during a count. the member `votes`
// represents the integer value of the votes transferred to the
// candidate, after the application of the transfer value
#[derive(Debug)]
pub struct BundleTransaction {
    pub ballot_states: Vec<BallotState>,
    pub transfer_value: u32,
    pub votes: u32
}

#[derive(Debug)]
pub struct CandidateData {
    pub count: usize,
    pub tickets: Vec<String>,
    pub ticket_candidates: HashMap<String, Vec<CandidateIndex>>
}
