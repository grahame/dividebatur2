use defs::*;
use num::rational::Ratio;
use num::BigInt;
use num::{FromPrimitive, ToPrimitive};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
/// the outcome of a count
pub enum CountOutcome {
    /// the engine has reached an endpoint
    CountComplete(usize, CountState),
    /// the engine has not reached an endpoint
    CountContinues(usize, CountState),
}

// these actions are in precedence order, low-to-high
#[derive(Debug)]
enum CountAction {
    FirstCount,
    ExclusionDistribution(CandidateIndex, Ratio<BigInt>),
    ElectionDistribution(CandidateIndex, Ratio<BigInt>),
}

#[derive(Debug, Clone)]
/// a summary of the state of the count, after a given
/// round of counting. referred to when breaking ties
/// for candidate election of exclusion
pub struct CountState {
    pub votes_per_candidate: HashMap<CandidateIndex, u32>,
    pub papers_per_candidate: HashMap<CandidateIndex, u32>,
    pub votes_exhausted: u32,
    pub papers_exhausted: u32,
}

#[derive(Debug)]
/// the number of papers and votes exhausted as the result of a distribution
struct DistributionOutcome {
    votes_exhausted: u32,
    papers_exhausted: u32,
}

/// Single Transferable Vote count engine
pub struct CountEngine {
    pub vacancies: u32,
    pub total_papers: u32,
    pub quota: u32,
    candidates: CandidateData,
    /// the `BundleTransaction`s held by each candidate
    candidate_bundle_transactions: HashMap<CandidateIndex, CandidateBundleTransactions>,
    count_states: Vec<CountState>,
    results: CountResults,
    actions_pending: VecDeque<CountAction>,
    automation: VecDeque<usize>,
}

#[derive(Debug)]
/// all bundle transactions held by a candidate in a given round of the count
struct CandidateBundleTransactions(Vec<BundleTransaction>);

impl CandidateBundleTransactions {
    fn total_votes(&self) -> u32 {
        self.0.iter().map(|bt| bt.votes).sum()
    }
    fn total_papers(&self) -> u32 {
        self.0.iter().map(|bt| bt.papers).sum()
    }
    fn new() -> CandidateBundleTransactions {
        CandidateBundleTransactions(Vec::new())
    }
}

impl CountEngine {
    /// determine the election quota
    fn determine_quota(total_papers: u32, vacancies: u32) -> u32 {
        (total_papers / (vacancies + 1)) + 1
    }

    /// apply transfer value to a number of votes. Rounds down.
    fn apply_transfer_value(transfer_value: &Ratio<BigInt>, votes: u32) -> u32 {
        let v: Ratio<BigInt> = Ratio::from_integer(FromPrimitive::from_u32(votes).unwrap());
        let vr = (transfer_value * v).to_integer();
        ToPrimitive::to_u32(&vr).unwrap()
    }

    /// bundle ballots together based upon the currently active preference. incrementally updates
    /// `self.candidate_bundle_transactions` with these papers, which must have been removed from
    /// this structure if they are being distributed as the result of an exclusion or election
    fn bundle_ballot_states(
        &mut self,
        ballot_states: Vec<BallotState>,
        transfer_value: Ratio<BigInt>,
    ) {
        let mut by_candidate: HashMap<CandidateIndex, Vec<BallotState>> = HashMap::new();
        for ballot_state in ballot_states.into_iter() {
            let candidate_id = match ballot_state.current_preference() {
                Some(p) => p,
                None => panic!("informal ballot in initial ballots"),
            };
            let v = by_candidate.entry(candidate_id).or_insert_with(Vec::new);
            v.push(ballot_state);
        }
        for (candidate_id, ballot_states) in by_candidate.drain() {
            let t = self
                .candidate_bundle_transactions
                .entry(candidate_id)
                .or_insert_with(CandidateBundleTransactions::new);
            let papers = ballot_states.iter().map(|bs| bs.count).sum();
            let bt = BundleTransaction {
                ballot_states,
                transfer_value: transfer_value.clone(),
                papers,
                votes: CountEngine::apply_transfer_value(&transfer_value, papers),
            };
            t.0.push(bt);
        }
    }

    /// distribute bundle transactions as the result of an election or an exclusion.
    /// moves the state of each bundle transaction on to the next preference, then
    /// calls on to `bundle_ballot_states`
    fn distribute_bundle_transactions(
        &mut self,
        bundle_transactions: Vec<BundleTransaction>,
        transfer_value: Ratio<BigInt>,
    ) -> DistributionOutcome {
        // the bundle_transactions should already have been removed from the previous holder
        let mut ballot_states = Vec::new();
        let initial_papers: u32 = bundle_transactions.iter().map(|bs| bs.papers).sum();

        for mut bundle_transaction in bundle_transactions {
            bundle_transaction
                .ballot_states
                .par_iter_mut()
                .for_each(|ballot_state| {
                    ballot_state.goto_next_preference(&self.results);
                });
            for ballot_state in bundle_transaction.ballot_states {
                if ballot_state.alive() {
                    ballot_states.push(ballot_state);
                }
            }
        }
        let papers_exhausted = initial_papers - ballot_states.len() as u32;
        let votes_exhausted = CountEngine::apply_transfer_value(&transfer_value, papers_exhausted);
        self.bundle_ballot_states(ballot_states, transfer_value);
        DistributionOutcome {
            votes_exhausted,
            papers_exhausted,
        }
    }

    /// Create a new STV count engine
    ///
    /// # Arguments
    ///
    /// * `vacancies` - the number of candidates to elect. For the Australian senate, this is `12` (full) or `6` (half)
    /// * `candidates` - the candidates running
    /// * `ballot_states` - the ballots cast
    /// * `automation` - a queue of automation outcomes
    pub fn new(
        vacancies: u32,
        candidates: CandidateData,
        ballot_states: Vec<BallotState>,
        automation: VecDeque<usize>,
    ) -> CountEngine {
        let total_papers = ballot_states.iter().map(|bs| bs.count).sum();
        let mut engine = CountEngine {
            candidates,
            vacancies,
            automation,
            total_papers,
            candidate_bundle_transactions: HashMap::new(),
            count_states: Vec::new(),
            quota: CountEngine::determine_quota(total_papers, vacancies),
            results: CountResults::new(),
            actions_pending: VecDeque::new(),
        };
        engine.bundle_ballot_states(
            ballot_states,
            Ratio::from_integer(FromPrimitive::from_u32(1).unwrap()),
        );
        engine.push_action(CountAction::FirstCount);
        engine
    }

    #[allow(dead_code)]
    pub fn print_debug(&self) {
        println!(
            "-- CountEngine::print_debug (round {}) --",
            self.count_states.len()
        );
        println!("Candidates: {}", self.candidates.count);
        println!("Total papers: {}", self.total_papers);
        println!("Quota: {}", self.quota);
        println!("Candidate totals:");
        let mut cbt: Vec<(&CandidateIndex, (u32, u32))> = self
            .candidate_bundle_transactions
            .iter()
            .map(|a| (a.0, (a.1.total_votes(), a.1.total_papers())))
            .collect();
        cbt.sort_by(|a, b| b.1.cmp(&a.1));
        for (candidate_id, (votes, papers)) in cbt {
            println!(
                "    {} votes for candidate {} ({}) [{} papers]",
                votes,
                self.candidates.get_name(*candidate_id),
                self.candidates.get_party(*candidate_id),
                papers
            );
        }
        println!(
            "Candidates elected: {}",
            self.candidates.vec_names(self.results.get_elected())
        );
        println!(
            "Candidates excluded: {}",
            self.candidates.vec_names(self.results.get_excluded())
        );
    }

    fn determine_elected_candidates(&mut self) -> Vec<CandidateIndex> {
        // determine all candidates whose vote total is over the threshold; bin by
        // the number of votes they are holding, so we can determine any ties
        let mut votes_candidate: HashMap<u32, Vec<CandidateIndex>> = HashMap::new();
        for (candidate_id, cbt) in self.candidate_bundle_transactions.iter() {
            if self.results.candidate_is_inactive(*candidate_id) {
                continue;
            }
            let votes = cbt.total_votes();
            if votes > self.quota {
                let v = votes_candidate
                    .entry(cbt.total_votes())
                    .or_insert_with(Vec::new);
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
        self.actions_pending.push_back(action);
    }

    fn elect(&mut self, candidate: CandidateIndex, state: &CountState) {
        if self.results.candidate_is_inactive(candidate) {
            panic!("Election of a candidate who was already excluded or elected.");
        }
        println!(
            "Elected candidate: {}",
            self.candidates.vec_names(&[candidate])
        );
        self.results.candidate_elected(candidate);
        let candidate_votes = state.votes_per_candidate[&candidate];
        let candidate_papers = state.papers_per_candidate[&candidate];
        let excess_votes = if candidate_votes > self.quota {
            candidate_votes - self.quota
        } else {
            0
        };
        let transfer_value = Ratio::from_integer(FromPrimitive::from_u32(excess_votes).unwrap())
            / Ratio::from_integer(FromPrimitive::from_u32(candidate_papers).unwrap());
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
            votes_exhausted,
        }
    }

    fn process_election_distribution(
        &mut self,
        candidate: CandidateIndex,
        transfer_value: Ratio<BigInt>,
    ) {
        let bundles_to_distribute = self
            .candidate_bundle_transactions
            .remove(&candidate)
            .unwrap()
            .0;
        self.distribute_bundle_transactions(bundles_to_distribute, transfer_value);
    }

    fn process_exclusion_distribution(
        &mut self,
        candidate: CandidateIndex,
        transfer_value: Ratio<BigInt>,
    ) {
        let current_bundles = self
            .candidate_bundle_transactions
            .remove(&candidate)
            .unwrap()
            .0;
        let mut bundles_to_distribute = Vec::new();
        let mut bundles_to_hold = Vec::new();
        for bundle in current_bundles {
            if bundle.transfer_value == transfer_value {
                bundles_to_distribute.push(bundle);
            } else {
                bundles_to_hold.push(bundle);
            }
        }
        // put the remaining bundles, if any, back in
        if !bundles_to_hold.is_empty() {
            self.candidate_bundle_transactions
                .insert(candidate, CandidateBundleTransactions(bundles_to_hold));
        }
        self.distribute_bundle_transactions(bundles_to_distribute, transfer_value);
    }

    fn find_tie_breaker(&self, candidates: &[CandidateIndex]) -> Option<Vec<(CandidateIndex)>> {
        // look back through previous counts, looking for a round where the votes of each of the candidates
        // are distinct. if found, returns the candidates in ascending vote order
        for (idx, count_state) in self.count_states.iter().enumerate().rev().skip(1) {
            let mut candidate_votes = Vec::new();
            let mut vote_set = HashSet::new();
            for candidate in candidates {
                let votes = count_state.votes_per_candidate[candidate];
                candidate_votes.push((*candidate, votes));
                vote_set.insert(votes);
            }
            if vote_set.len() == candidates.len() {
                println!("find tie breaker: match found in round {}", idx);
                candidate_votes.sort_by_key(|&(_, v)| v);
                return Some(candidate_votes.drain(..).map(|(c, _)| c).collect());
            }
        }
        None
    }

    fn exclude_a_candidate(&mut self, count_state: &CountState) {
        let mut votes_eligible_candidate = Vec::new();
        for (candidate, votes) in count_state.votes_per_candidate.iter() {
            if self.results.candidate_is_inactive(*candidate) {
                continue;
            }
            votes_eligible_candidate.push((*candidate, *votes));
        }
        assert!(!votes_eligible_candidate.is_empty());
        let min_votes = votes_eligible_candidate
            .iter()
            .map(|&(_, v)| v)
            .min()
            .unwrap();
        let exclusion_candidates: Vec<CandidateIndex> = votes_eligible_candidate
            .drain(..)
            .filter(|&(_, v)| v == min_votes)
            .map(|(c, _)| c)
            .collect();

        let possibilities = exclusion_candidates.len();
        let to_exclude = if possibilities == 0 {
            panic!("No candidates left for exclusion, yet we're trying to exclude");
        } else if possibilities == 1 {
            exclusion_candidates[0]
        } else {
            match self.find_tie_breaker(&exclusion_candidates) {
                Some(tie_broken_candidates) => tie_broken_candidates[0],
                None => {
                    let auto = self.automation.pop_front().unwrap();
                    exclusion_candidates[auto]
                }
            }
        };
        println!(
            "exclude_a_candidate: {}",
            self.candidates.vec_names(&exclusion_candidates)
        );

        self.results.candidate_excluded(to_exclude);

        let mut transfer_values = HashSet::new();
        {
            let bundle_transactions = &self.candidate_bundle_transactions[&to_exclude].0;
            for bundle_transaction in bundle_transactions.iter() {
                transfer_values.insert(bundle_transaction.transfer_value.clone());
            }
        }
        let mut transfer_values: Vec<Ratio<BigInt>> = transfer_values.drain().collect();
        transfer_values.sort();
        transfer_values.reverse();
        for transfer_value in transfer_values {
            self.push_action(CountAction::ExclusionDistribution(
                to_exclude,
                transfer_value,
            ));
        }
    }

    pub fn count(&mut self) -> CountOutcome {
        println!("-- START ROUND {} --", self.count_states.len() + 1);
        println!();

        let votes_exhausted = 0;
        let papers_exhausted = 0;

        // count votes, once (a single 'round')
        let action = self.actions_pending.pop_front().unwrap();
        match action {
            CountAction::FirstCount => {
                // we don't need to do anything on the first count
                println!("Action: first count");
            }
            CountAction::ExclusionDistribution(candidate, transfer_value) => {
                println!(
                    "Action: exclusion distribution of papers from candidate {} with transfer value {}",
                    self.candidates.vec_names(&[candidate]), transfer_value);
                self.process_exclusion_distribution(candidate, transfer_value);
            }
            CountAction::ElectionDistribution(candidate, transfer_value) => {
                println!(
                    "Action: election distribution of candidate {}",
                    self.candidates.vec_names(&[candidate])
                );
                self.process_election_distribution(candidate, transfer_value);
            }
        }

        // determine count totals
        let count_state = self.build_count_state(votes_exhausted, papers_exhausted);
        self.count_states.push(count_state.clone());

        // has anyone been elected in this count?
        let newly_elected = self.determine_elected_candidates();
        for candidate in newly_elected {
            self.elect(candidate, &count_state);
            if self.results.number_elected() == self.vacancies {
                return CountOutcome::CountComplete(self.count_states.len(), count_state);
            }
        }

        // are we done? check the various termination procedures from the Act
        if self.actions_pending.is_empty() {
            let mut continuing_candidates: Vec<CandidateIndex> = count_state
                .votes_per_candidate
                .keys()
                .filter(|c| !self.results.candidate_is_inactive(**c))
                .cloned()
                .collect();
            continuing_candidates.sort_by_key(|c| count_state.votes_per_candidate[c]);
            let remaining_vacancies = self.vacancies - self.results.number_elected();
            // section 273(18); if we're down to N candidates in the running, with N vacancies, the remaining candidates are elected
            if continuing_candidates.len() as u32 == remaining_vacancies {
                for candidate in continuing_candidates.iter().rev() {
                    self.elect(*candidate, &count_state);
                }
                return CountOutcome::CountComplete(self.count_states.len(), count_state);
            }
            // section 273(17); if we're down to two candidates in the running, the candidate with the highest number of votes wins - even
            // if they don't have a quota
            if continuing_candidates.len() == 2 {
                let a = continuing_candidates[0];
                let b = continuing_candidates[1];
                if count_state.votes_per_candidate[&a] == count_state.votes_per_candidate[&b] {
                    panic!("Must manually choose for tie on last spot.");
                } else {
                    self.elect(b, &count_state);
                    return CountOutcome::CountComplete(self.count_states.len(), count_state);
                }
            }
        }

        // if we don't have anything pending (exclusion or election), then it's
        // time to exclude a candidate
        if self.actions_pending.is_empty() {
            self.exclude_a_candidate(&count_state);
        }

        CountOutcome::CountContinues(self.count_states.len(), count_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_transfer_value() {
        let a: Ratio<BigInt> = Ratio::from_integer(FromPrimitive::from_u32(1).unwrap())
            / Ratio::from_integer(FromPrimitive::from_u32(3).unwrap());
        println!("{:?}", CountEngine::apply_transfer_value(&a, 1));
        assert!(CountEngine::apply_transfer_value(&a, 0) == 0);
        assert!(CountEngine::apply_transfer_value(&a, 1) == 0);
        assert!(CountEngine::apply_transfer_value(&a, 2) == 0);
        assert!(CountEngine::apply_transfer_value(&a, 3) == 1);
        assert!(CountEngine::apply_transfer_value(&a, 4) == 1);
        assert!(CountEngine::apply_transfer_value(&a, 5) == 1);
        assert!(CountEngine::apply_transfer_value(&a, 6) == 2);
    }
}
