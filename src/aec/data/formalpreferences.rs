//
// Parse the formal preferences CSV file
// Example file: http://results.aec.gov.au/20499/Website/External/aec-senate-formalpreferences-20499-NT.zip
//

extern crate csv;

use std::error::Error;
use std::fs::File;
use rayon::prelude::*;

#[derive(Debug,Deserialize)]
pub struct AECFormalPreferencesRow {
    #[serde(rename = "Preferences")]
    preferences: String,
}

fn parse_preferences(raw_preferences: &String, candidates: &::CandidateData) -> Vec<::CandidateIndex> {
    let ticket_count = candidates.ticket_candidates.len();

    let mut atl_buf: Vec<(::PreferenceForGroup, ::GroupIndex)> = Vec::with_capacity(ticket_count);
    let mut btl_buf: Vec<(::PreferenceForCandidate, ::CandidateIndex)> = Vec::with_capacity(candidates.count);
    let mut form_buf: Vec<::CandidateIndex> = Vec::with_capacity(candidates.count);

    atl_buf.clear();
    btl_buf.clear();

    for (pref_idx, pref_str) in raw_preferences.split(",").enumerate() {
        let pref_v: u32 = if pref_str == "" {
            continue
        } else if pref_str == "*" || pref_str == "/" {
            1
        } else {
            pref_str.parse::<u32>().unwrap()
        };

        if pref_idx < ticket_count {
            atl_buf.push((::PreferenceForGroup(pref_v as u8), ::GroupIndex(pref_idx as u8)));
        } else {
            btl_buf.push((::PreferenceForCandidate(pref_v as u8), ::CandidateIndex(pref_idx as u8)));
        }
    }

    atl_buf.sort();
    btl_buf.sort();

    form_buf.clear();
    for idx in 0..btl_buf.len() {
        let (pref, candidate_id) = btl_buf[idx];
        // the preference at this index must be the index plus 1
        if pref != ::PreferenceForCandidate((idx + 1) as u8) {
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

    for idx in 0..atl_buf.len() {
        let (pref, group_id) = atl_buf[idx];
        // the preference at this index must be the index plus 1
        if pref != ::PreferenceForGroup((idx + 1) as u8) {
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

pub fn load(filename: &str, candidates: &::CandidateData) -> Result<Vec<Vec<::CandidateIndex>>, Box<Error>> {
    let f = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(f);
    let hunk_size = 1024;

    let mut raw_preferences = Vec::new();
    let mut work_buf = Vec::new();
    let mut forms: Vec<Vec<::CandidateIndex>> = Vec::new();

    let process = |w: &mut Vec<String>, r: &mut Vec<Vec<::CandidateIndex>>| {
        let mut partial: Vec<Vec<::CandidateIndex>> = w.par_iter().map(|p| parse_preferences(p, candidates)).collect();
        r.append(&mut partial);
        w.clear();
    };

    for (idx, result) in rdr.deserialize().enumerate() {
        if idx == 0 {
            continue;
        }
        let record: AECFormalPreferencesRow = result?;
        work_buf.push(record.preferences);

        if work_buf.len() >= hunk_size {
            process(&mut work_buf, &mut forms);
        }
    }
    process(&mut work_buf, &mut forms);

    for (idx, result) in rdr.deserialize().enumerate() {
        // the first row is always garbage (heading '----' markers)
        if idx == 0 {
            continue;
        }
        let record: AECFormalPreferencesRow = result?;
        raw_preferences.push(record.preferences);
    }

    Ok(forms)
}