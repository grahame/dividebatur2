use std::collections::HashMap;
use defs::*;
use num::BigInt;
use num::FromPrimitive;
use num::rational::{Ratio};

#[derive(Debug)]
pub enum CountOutcome {
    CountComplete(usize, CountState),
    CountContinues(usize, CountState)
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord)]
// these actions are in precedence order, low-to-high
enum CountAction {
    FirstCount,
    ExclusionDistribution(CandidateIndex),
    ElectionDistribution(CandidateIndex, Ratio<BigInt>),
}

#[derive(Debug, Clone)]
pub struct CountState {
    pub votes_per_candidate: HashMap<CandidateIndex, u32>,
    pub papers_per_candidate: HashMap<CandidateIndex, u32>,
    pub votes_exhausted: u32,
    pub papers_exhausted: u32,
}

#[derive(Debug)]
pub struct CountEngine {
    candidates: u32,
    vacancies: u32,
    candidate_bundle_transactions: CandidateToBundleTransaction,
    total_papers: u32,
    count_states: Vec<CountState>,
    quota: u32,
    elected: Vec<CandidateIndex>,
    excluded: Vec<CandidateIndex>,
    actions_pending: Vec<(CountAction, usize)>
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
    fn total_papers(&self) -> u32 {
        self.bundle_transactions.iter().map(|bt| bt.papers).sum()
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
                papers: votes,
            };
            t.bundle_transactions.push(bt);
        }
        let mut engine = CountEngine {
            candidates: candidates,
            vacancies: vacancies,
            candidate_bundle_transactions: ctbt,
            total_papers: total_papers,
            count_states: Vec::new(),
            quota: CountEngine::determine_quota(total_papers, vacancies),
            elected: Vec::new(),
            excluded: Vec::new(),
            actions_pending: Vec::new(),
        };
        engine.push_action(CountAction::FirstCount);
        engine
    }

    pub fn print_debug(&self, cd: &CandidateData) {
        println!("-- CountEngine::print_debug (round {}) --", self.count_states.len());
        println!("Candidates: {}", self.candidates);
        println!("Total papers: {}", self.total_papers);
        println!("Quota: {}", self.quota);
        println!("Candidate totals:");
        let mut cbt: Vec<(&CandidateIndex, &CandidateBundleTransactions)> = self.candidate_bundle_transactions.iter().collect();
        cbt.sort_by(|a, b| a.0.cmp(b.0));
        for (candidate_id, cbts) in cbt {
            let a: u32 = cbts.total_votes();
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

    fn push_action(&mut self, action: CountAction) {
        // we need to maintain the list of actions to perform in a precedence order,
        // by type of action, and then the order in which they were added to the queue
        // there's probably a more idiomatic Rust way to do this.
        let offset = self.actions_pending.len();
        println!("Action pushed: {:?}", action);
        self.actions_pending.push((action, offset));
        self.actions_pending.sort();
        self.actions_pending.reverse();
        println!("Actions pending: {:?}", self.actions_pending);
    }

    fn pop_action(&mut self) -> CountAction {
        let (action, _) = self.actions_pending.pop().unwrap();
        action
    }

    fn elect(&mut self, candidate: CandidateIndex, state: &CountState) {
        if self.elected.contains(&candidate) { 
            panic!("Candidate elected twice");
        }
        println!("Elected candidate: {:?}", candidate);
        self.elected.push(candidate);
        let candidate_votes = *state.votes_per_candidate.get(&candidate).unwrap();
        let candidate_papers = *state.papers_per_candidate.get(&candidate).unwrap();
        let excess_votes = if candidate_votes > self.quota {
            candidate_votes - self.quota
        } else {
            0
        };
        let transfer_value = Ratio::from_integer(FromPrimitive::from_u32(excess_votes).unwrap()) / Ratio::from_integer(FromPrimitive::from_u32(candidate_papers).unwrap());
        self.push_action(CountAction::ElectionDistribution(candidate, transfer_value));
    }

    fn build_count_state(&self, votes_exhausted: u32, papers_exhausted: u32) -> CountState {
        let mut vpc: HashMap<CandidateIndex, u32> = HashMap::new();
        let mut ppc: HashMap<CandidateIndex, u32> = HashMap::new();
        for (candidate_id, cbts) in self.candidate_bundle_transactions.iter() {
            vpc.insert(*candidate_id, cbts.total_votes());
            ppc.insert(*candidate_id, cbts.total_papers());
        }
        CountState {
            votes_per_candidate: vpc,
            papers_per_candidate: ppc,
            papers_exhausted,
            votes_exhausted
        }
    }

    pub fn count(&mut self) -> CountOutcome {
        let votes_exhausted = 0;
        let papers_exhausted = 0;

        // count votes, once (a single 'round')
        match self.pop_action() {
            CountAction::FirstCount => {
                // we don't need to do anything on the first count
            },
            CountAction::ExclusionDistribution(candidate) => {
            }
            CountAction::ElectionDistribution(candidate, transfer_value) => {
            }
        }

        // action execution to come

        // determine count totals
        let count_state = self.build_count_state(votes_exhausted, papers_exhausted);
        self.count_states.push(count_state.clone());

        // has anyone been elected in this count?
        let newly_elected = self.determine_elected_candidates();
        for candidate in newly_elected {
            self.elect(candidate, &count_state);
            if self.elected.len() as u32 == self.vacancies {
                return CountOutcome::CountComplete(self.count_states.len(), count_state);
            }
        }
        return CountOutcome::CountContinues(self.count_states.len(), count_state);
    }
}
