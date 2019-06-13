use aec;
use defs::*;

pub fn load_candidate_data(
    candidates: Vec<aec::data::candidates::AECAllCandidateRow>,
) -> CandidateData {
    let mut names = Vec::new();
    let mut parties = Vec::new();

    let mut current_ticket = String::from("");
    let mut tickets = Vec::new();

    // NB: the Candidate Rows are sorted into ballot paper order
    for (idx, candidate) in candidates.iter().enumerate() {
        names.push(format!(
            "{}, {}",
            candidate.surname, candidate.ballot_given_nm
        ));
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
        names,
        parties,
        tickets,
    }
}
