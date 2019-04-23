//
// Parse the formal preferences CSV file
// Example file: http://results.aec.gov.au/20499/Website/External/aec-senate-formalpreferences-20499-NT.zip
//

extern crate csv;
extern crate flate2;

use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use rayon::prelude::*;
use defs::*;

#[derive(Debug, Deserialize)]
struct AECFormalPreferencesRow {
    #[serde(rename = "Preferences")] preferences: String,
}

// a voter's numerical preference for a candidate
// if valid, it ranges from 1..N where N is the number of candidates
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct CandidatePreference(pub u8);

// a voter's numerical preference for a group
// if valid, it ranges from 1..N where N is the number of groups
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct GroupPreference(pub u8);

fn parse_preferences(raw_preferences: &String, candidates: &CandidateData) -> Option<Vec<CandidateIndex>> {
    let ticket_count = candidates.tickets.len();

    let mut atl_buf: Vec<(GroupPreference, GroupIndex)> = Vec::with_capacity(ticket_count);
    let mut btl_buf: Vec<(CandidatePreference, CandidateIndex)> =
        Vec::with_capacity(candidates.count);
    let mut form_buf: Vec<CandidateIndex> = Vec::with_capacity(candidates.count);

    for (pref_idx, pref_str) in raw_preferences.split(",").enumerate() {
        let pref_v: u8 = if pref_str == "" {
            continue;
        } else if pref_str == "*" || pref_str == "/" {
            1
        } else {
            pref_str.parse::<u8>().unwrap()
        };

        if pref_idx < ticket_count {
            atl_buf.push((GroupPreference(pref_v), GroupIndex(pref_idx as u8)));
        } else {
            btl_buf.push((
                CandidatePreference(pref_v),
                CandidateIndex((pref_idx - ticket_count) as u8),
            ));
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
        return None; // form_buf.clone();
    }

    // we don't have a valid BTL form, validate and expand above-the-line
    // preferences
    form_buf.clear();

    atl_buf.sort();
    for idx in 0..atl_buf.len() {
        let (pref, group_index) = atl_buf[idx];
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
        form_buf.extend(&candidates.tickets[group_index.0 as usize]);
    }

    if form_buf.len() == 0 {
        return None;
    }

    return Some(form_buf);
}

pub fn load(filename: &str, candidates: &CandidateData) -> Result<Vec<BallotState>, Box<Error>> {
    let f = File::open(filename)?;
    let gf = flate2::read::GzDecoder::new(f);
    let mut rdr = csv::Reader::from_reader(gf);

    let mut keys: HashMap<Vec<CandidateIndex>, u32> = HashMap::new();
    for (idx, result) in rdr.deserialize().enumerate() {
        if idx == 0 {
            continue;
        }
        let record: AECFormalPreferencesRow = result?;
        let prefs = parse_preferences(&record.preferences, candidates);
        match prefs {
            Some(form) => {
                let counter = keys.entry(form.clone()).or_insert(0);
                *counter += 1;
            }
            None => {}
        }
    }

    let r = keys.drain()
        .map(|(form, count)| BallotState {
            form,
            count,
            active_preference: 0,
        })
        .collect();
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let cd = CandidateData {
            count: 19,
            names: vec![],
            parties: vec![],
            tickets: vec![
                vec![CandidateIndex(0), CandidateIndex(1)],
                vec![CandidateIndex(2), CandidateIndex(3)],
                vec![CandidateIndex(4), CandidateIndex(5)],
                vec![CandidateIndex(6), CandidateIndex(7)],
                vec![CandidateIndex(8), CandidateIndex(9)],
                vec![CandidateIndex(10), CandidateIndex(11)],
                vec![CandidateIndex(12), CandidateIndex(13)],
            ],
        };
        let line = String::from("5,7,3,4,1,2,6,5,6,11,12,3,4,9,10,1,2,2,3,7,8,,,,,");
        let a = parse_preferences(&line, &cd);
        println!("{:?}", a);
    }
}
