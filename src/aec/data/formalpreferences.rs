//
// Parse the formal preferences CSV file
// Example file: http://results.aec.gov.au/20499/Website/External/aec-senate-formalpreferences-20499-NT.zip
//

extern crate csv;

use std::error::Error;
use std::fs::File;

#[derive(Debug,Deserialize)]
pub struct AECFormalPreferencesRow {
    #[serde(rename = "ElectorateNm")]
    electorate_nm: String,
    #[serde(rename = "VoteCollectionPointNm")]
    vote_collection_point_nm: String,
    #[serde(rename = "VoteCollectionPointId")]
    vote_collection_point_id: i32,
    #[serde(rename = "BatchNo")]
    batch_no: i32,
    #[serde(rename = "PaperNo")]
    paper_no: i32,
    #[serde(rename = "Preferences")]
    preferences: String,
}

pub fn load(filename: &str) -> Result<Vec<AECFormalPreferencesRow>, Box<Error>> {
    let f = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(f);
    let mut rows: Vec<AECFormalPreferencesRow> = Vec::new();
    for (idx, result) in rdr.deserialize().enumerate() {
        // the first row is always garbage (heading '----' markers)
        let record: AECFormalPreferencesRow = match result {
            Ok(v) => v,
            Err(e) => if idx == 0 {
                continue
            } else {
                return Err(Box::new(e));
            }
        };
        rows.push(record);
    }
    Ok((rows))
}