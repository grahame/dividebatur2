use defs::*;
use engine::*;
use aec;
use std::collections::VecDeque;

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
        names: names,
        parties: parties,
        tickets: tickets,
    }
}

fn run_state(state: &str, vacancies: u32) {
    println!(" -- LOAD: {}", state);

    let candidates = match aec::data::candidates::load(
        "dividebatur-aec/fed2016/common/aec-senate-candidateinformation-20499.csv",
        state,
    ) {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read candidates file: {:?}", error);
        }
    };
    let cd = load_candidate_data(candidates);

    let prefpath = format!(
        "dividebatur-aec/fed2016/{}/data/aec-senate-formalpreferences-20499-{}.csv.gz",
        state.to_ascii_lowercase(),
        state.to_ascii_uppercase()
    );

    let ballot_states = match aec::data::formalpreferences::load(&prefpath[..], &cd) {
        Ok(data) => data,
        Err(error) => {
            panic!("Couldn't read formal preferences file: {:?}", error);
        }
    };

    println!(
        "{} unique bundle states at commencement of count.",
        ballot_states.len()
    );

    let mut automation = VecDeque::new();
    automation.push_back(0);
    let mut engine = CountEngine::new(vacancies, cd, ballot_states, automation);
    while {
        let outcome = engine.count();
        match outcome {
            CountOutcome::CountComplete(nrounds, state) => {
                // engine.print_debug();
                println!("{:?}", state);
                println!("Election complete after {} rounds of counting.", nrounds);
                false
            }
            CountOutcome::CountContinues(_, _) => {
                // engine.print_debug();
                true
            }
        }
    } {}
}

pub fn run() {
    let australia = vec![
        (2, "ACT"),
        (12, "NSW"),
        (2, "NT"),
        (12, "QLD"),
        (12, "SA"),
        (12, "TAS"),
        (12, "VIC"),
        (12, "WA"),
    ];
    for (vacancies, state) in australia {
        run_state(state, vacancies);
    }
}
