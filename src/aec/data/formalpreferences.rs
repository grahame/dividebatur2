//
// Parse the formal preferences CSV file
// Example file: http://results.aec.gov.au/20499/Website/External/aec-senate-formalpreferences-20499-NT.zip
//

extern crate csv;

use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use rayon::prelude::*;
use defs::*;

#[derive(Debug,Deserialize)]
struct AECFormalPreferencesRow {
    #[serde(rename = "Preferences")]
    preferences: String,
}

// a voter's numerical preference for a candidate
// if valid, it ranges from 1..N where N is the number of candidates
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct CandidatePreference(pub u8);

// a voter's numerical preference for a group
// if valid, it ranges from 1..N where N is the number of groups
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct GroupPreference(pub u8);

fn parse_preferences(raw_preferences: &String, candidates: &CandidateData) -> Vec<CandidateIndex> {
    let ticket_count = candidates.ticket_candidates.len();

    let mut atl_buf: Vec<(GroupPreference, GroupIndex)> = Vec::with_capacity(ticket_count);
    let mut btl_buf: Vec<(CandidatePreference, CandidateIndex)> = Vec::with_capacity(candidates.count);
    let mut form_buf: Vec<CandidateIndex> = Vec::with_capacity(candidates.count);

    for (pref_idx, pref_str) in raw_preferences.split(",").enumerate() {
        let pref_v: u32 = if pref_str == "" {
            continue
        } else if pref_str == "*" || pref_str == "/" {
            1
        } else {
            pref_str.parse::<u32>().unwrap()
        };

        if pref_idx < ticket_count {
            atl_buf.push((GroupPreference(pref_v as u8), GroupIndex(pref_idx as u8)));
        } else {
            btl_buf.push((CandidatePreference(pref_v as u8), CandidateIndex((pref_idx - ticket_count) as u8)));
        }
    }

    // Validate below-the-line preferences. If these are valid, they take
    // precedence over any above-the-line preferences.
    btl_buf.sort();
    for idx in 0..btl_buf.len() {
        let (pref, candidate_id) = btl_buf[idx];
        // the preference at this index must be the index plus 1
        if pref != CandidatePreference((idx + 1) as u8) {
            break;
        }
        // look ahead: we can't have double preferences
        if idx < (btl_buf.len() - 1) {
            let (next_pref, _) = btl_buf[idx + 1];
            if next_pref == pref {
                break;
            }
        }
        form_buf.push(candidate_id);
    }

    // if we have at least six BTL prefrences, we have a valid form
    if form_buf.len() >= 6 {
        return form_buf.clone();
    }

    // Validate and expand above-the-line preferences.
    atl_buf.sort();
    for idx in 0..atl_buf.len() {
        let (pref, group_id) = atl_buf[idx];
        // the preference at this index must be the index plus 1
        if pref != GroupPreference((idx + 1) as u8) {
            break;
        }
        // look ahead: we can't have double preferences
        if idx < (atl_buf.len() - 1) {
            let (next_pref, _) = atl_buf[idx + 1];
            if next_pref == pref {
                break;
            }
        }
        // valid ATL preference. push this form into the form_buf!
        let group_name = &candidates.tickets[group_id.0 as usize];
        form_buf.extend(&candidates.ticket_candidates[group_name]);
    }

    assert!(form_buf.len() > 0);

    return form_buf;
}

pub fn load(filename: &str, candidates: &CandidateData) -> Result<Vec<BallotState>, Box<Error>> {
    let f = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(f);
    let hunk_size = 1024;

    let mut work_buf = Vec::new();

    let process = |w: &mut Vec<String>, r: &mut HashMap<Vec<CandidateIndex>, u32>| {
        let partial: Vec<Vec<CandidateIndex>> = w.par_iter().map(|p| parse_preferences(p, candidates)).collect();
        for form in partial.iter() {
            let counter = r.entry(form.clone()).or_insert(0);
            *counter += 1;
        }
        w.clear();
    };

    let mut keys: HashMap<Vec<CandidateIndex>, u32> = HashMap::new();
    for (idx, result) in rdr.deserialize().enumerate() {
        if idx == 0 {
            continue;
        }
        let record: AECFormalPreferencesRow = result?;
        work_buf.push(record.preferences);

        if work_buf.len() >= hunk_size {
            process(&mut work_buf, &mut keys);
        }
    }
    process(&mut work_buf, &mut keys);

    let r = keys.drain().map(|(form, count)| BallotState {
        form,
        count,
        active_preference: 0
    }).collect();
    Ok(r)
}