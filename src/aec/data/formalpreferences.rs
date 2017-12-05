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

pub fn load(filename: &str, candidates: &::CandidateData) -> Result<Vec<Vec<u8>>, Box<Error>> {
    let f = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(f);
    let mut rows: Vec<Vec<u8>> = Vec::new();

    let n = 200;
    let tickets = candidates.ticket_candidates.len();
    // let mut pref_buf: Vec<u8> = Vec::with_capacity(tickets + (candidates.count as usize));

    for (idx, result) in rdr.deserialize().enumerate() {
        // the first row is always garbage (heading '----' markers)
        if idx == 0 {
            continue;
        }
        let record: AECFormalPreferencesRow = result?;

        for (pref_idx, pref_str) in record.preferences.split(",").enumerate() {
            let pref_v: u8 = if pref_str == "" {
                continue
            } else if pref_str == "*" || pref_str == "/" {
                1
            } else {
                pref_str.parse::<u8>().unwrap()
            };
        }
    }
    Ok((rows))
}