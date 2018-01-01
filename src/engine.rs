use std::collections::HashMap;
use defs::*;
use num::BigInt;
use num::{FromPrimitive,ToPrimitive};
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
struct DistributionOutcome {
    votes_exhausted: u32,
    papers_exhausted: u32,
}

pub struct CountEngine {
    candidates: CandidateData,
    vacancies: u32,
    candidate_bundle_transactions: CandidateToBundleTransactions,
    total_papers: u32,
    count_states: Vec<CountState>,
    quota: u32,
    elected: Vec<CandidateIndex>,
    excluded: Vec<CandidateIndex>,
    actions_pending: Vec<(CountAction, usize)>
}

// all bundle transactions held by a candidate, in a given round of the count
struct CandidateBundleTransactions {
    bundle_transactions: Vec<BundleTransaction>
}

type CandidateToBundleTransactions = HashMap<CandidateIndex, CandidateBundleTransactions>;

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

    fn apply_transfer_value(transfer_value: &Ratio<BigInt>, votes: u32) -> u32 {
        let v: Ratio<BigInt> = Ratio::from_integer(FromPrimitive::from_u32(votes).unwrap());
        let vr = (transfer_value * v).to_integer();
        let out = ToPrimitive::to_u32(&vr).unwrap();
        out
    }

    fn bundle_ballot_states(&mut self, ballot_states: Vec<BallotState>, transfer_value: Ratio<BigInt>) {
        let mut by_candidate: HashMap<CandidateIndex, Vec<BallotState>> = HashMap::new();
        for ballot_state in ballot_states.into_iter() {
            let candidate_id = match ballot_state.current_preference() {
                Some(p) => p,
                None => panic!("informal ballot in initial ballots")
            };
            let v = by_candidate.entry(candidate_id).or_insert(Vec::new());
            v.push(ballot_state);
        }
        for (candidate_id, ballot_states) in by_candidate.drain() {
            let t = self.candidate_bundle_transactions.entry(candidate_id).or_insert(CandidateBundleTransactions::new());
            let papers = ballot_states.iter().map(|bs| bs.count).sum();
            let bt = BundleTransaction {
                ballot_states: ballot_states,
                transfer_value: transfer_value.clone(),
                papers: papers,
                votes: CountEngine::apply_transfer_value(&transfer_value, papers)
            };
            t.bundle_transactions.push(bt);
        }
    }

    fn distribute_bundle_transactions(&mut self, bundle_transactions: &mut Vec<BundleTransaction>, transfer_value: Ratio<BigInt>) -> DistributionOutcome {
        // the bundle_transactions should already have been removed from the previous holder
        let mut papers_exhausted = 0;
        let mut ballot_states = Vec::new();

        for bundle_transaction in bundle_transactions {
            for mut ballot_state in bundle_transaction.ballot_states.drain(..) {
                if ballot_state.to_next_preference() {
                    ballot_states.push(ballot_state);
                } else {
                    papers_exhausted += 1;
                }
            }
        }
        let votes_exhausted = CountEngine::apply_transfer_value(&transfer_value, papers_exhausted);
        self.bundle_ballot_states(ballot_states, transfer_value);
        DistributionOutcome {
            votes_exhausted,
            papers_exhausted
        }
    }


    pub fn new(vacancies: u32, candidates: CandidateData, ballot_states: Vec<BallotState>) -> CountEngine {
        let total_papers = ballot_states.iter().map(|bs| bs.count).sum();
        let mut engine = CountEngine {
            candidates: candidates,
            vacancies: vacancies,
            candidate_bundle_transactions: HashMap::new(),
            total_papers: total_papers,
            count_states: Vec::new(),
            quota: CountEngine::determine_quota(total_papers, vacancies),
            elected: Vec::new(),
            excluded: Vec::new(),
            actions_pending: Vec::new(),
        };
        engine.bundle_ballot_states(ballot_states, Ratio::from_integer(FromPrimitive::from_u32(1).unwrap()));
        engine.push_action(CountAction::FirstCount);
        engine
    }

    pub fn print_debug(&self) {
        println!("-- CountEngine::print_debug (round {}) --", self.count_states.len());
        println!("Candidates: {}", self.candidates.count);
        println!("Total papers: {}", self.total_papers);
        println!("Quota: {}", self.quota);
        println!("Candidate totals:");
        let mut cbt: Vec<(&CandidateIndex, u32)> = self.candidate_bundle_transactions.iter().map(|a| (a.0, a.1.total_votes())).collect();
        cbt.sort_by(|a, b| b.1.cmp(&a.1));
        for (candidate_id, votes) in cbt {
            println!("    {} votes for candidate {} ({})", votes, self.candidates.get_name(*candidate_id), self.candidates.get_party(*candidate_id));
        }
        println!("Candidates elected: {}", self.candidates.vec_names(&self.elected));
        println!("Candidates excluded: {}", self.candidates.vec_names(&self.excluded));
    }

    fn determine_elected_candidates(&mut self) -> Vec<CandidateIndex> {
        // determine all candidates whose vote total is over the threshold; bin by
        // the number of votes they are holding, so we can determine any ties
        let mut votes_candidate: HashMap<u32, Vec<CandidateIndex>> = HashMap::new();
        for (candidate_id, cbt) in self.candidate_bundle_transactions.iter() {
            if self.elected.contains(candidate_id) || self.excluded.contains(candidate_id) {
                continue;
            }
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
        match self.actions_pending.pop() {
            Some(a) => {
                let (action, _) = a;
                action
            }
            None => {
                panic!("Ran out of actions - unreachable.");
            }
        }
    }

    fn elect(&mut self, candidate: CandidateIndex, state: &CountState) {
        if self.elected.contains(&candidate) { 
            panic!("Candidate elected twice");
        }
        println!("Elected candidate: {}", self.candidates.vec_names(&vec![candidate]));
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

    fn process_election_distribution(&mut self, candidate: CandidateIndex, transfer_value: Ratio<BigInt>, cs: &CountState) {
        let mut bundles_to_distribute = self.candidate_bundle_transactions.remove(&candidate).unwrap().bundle_transactions;
        self.distribute_bundle_transactions(&mut bundles_to_distribute, transfer_value);
    }

    pub fn count(&mut self) -> CountOutcome {
        let votes_exhausted = 0;
        let papers_exhausted = 0;

        // count votes, once (a single 'round')
        let action = self.pop_action();
        println!("Action is: {:?}", action);
        match action {
            CountAction::FirstCount => {
                // we don't need to do anything on the first count
            },
            CountAction::ExclusionDistribution(candidate) => {
            }
            CountAction::ElectionDistribution(candidate, transfer_value) => {
                let last_state = self.count_states[self.count_states.len() - 1].clone();
                self.process_election_distribution(candidate, transfer_value, &last_state);
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
