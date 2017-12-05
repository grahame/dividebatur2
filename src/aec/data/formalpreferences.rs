//
// Parse the formal preferences CSV file
// Example file: http://results.aec.gov.au/20499/Website/External/aec-senate-formalpreferences-20499-NT.zip
//

extern crate csv;

use std::error::Error;
use std::fs::File;

#[derive(Debug,Deserialize)]
pub struct AECFormalPreferencesRow {
    // These are in the CSV file, but we don't use them, so no point reading them:
    //   #[serde(rename = "ElectorateNm")]
    //   electorate_nm: String,
    //   #[serde(rename = "VoteCollectionPointNm")]
    //   vote_collection_point_nm: String,
    //   #[serde(rename = "VoteCollectionPointId")]
    //   vote_collection_point_id: i32,
    //   #[serde(rename = "BatchNo")]
    //   batch_no: i32,
    //   #[serde(rename = "PaperNo")]
    //   paper_no: i32,
    #[serde(rename = "Preferences")]
    preferences: String,
}

pub fn load(filename: &str, candidates: &::CandidateData) -> Result<Vec<Vec<::CandidateIndex>>, Box<Error>> {
    let f = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(f);

    let ticket_count = candidates.ticket_candidates.len();
    let mut atl_buf: Vec<(::PreferenceForGroup, ::GroupIndex)> = Vec::with_capacity(ticket_count);
    let mut btl_buf: Vec<(::PreferenceForCandidate, ::CandidateIndex)> = Vec::with_capacity(candidates.count);
    let mut form_buf: Vec<::CandidateIndex> = Vec::with_capacity(candidates.count);

    println!("reading formal preferences, {} candidates, {} groups", candidates.count, ticket_count);

    let mut forms: Vec<Vec<::CandidateIndex>> = Vec::new();

    for (idx, result) in rdr.deserialize().enumerate() {
        // the first row is always garbage (heading '----' markers)
        if idx == 0 {
            continue;
        }
        let record: AECFormalPreferencesRow = result?;

        atl_buf.clear();
        btl_buf.clear();

        for (pref_idx, pref_str) in record.preferences.split(",").enumerate() {
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
            forms.push(form_buf.clone());
            continue;
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

        // anything ATL is good
        if form_buf.len() > 0 {
            forms.push(form_buf.clone());
            continue;
        } else {
            println!("nothing formal");
            println!("{:?}", record.preferences);
            println!("{:?}", atl_buf);
            println!("{:?}", btl_buf);
            println!();
        }
    }
    Ok(forms)
}