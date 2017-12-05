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

pub fn load(filename: &str) -> Result<Vec<Vec<u8>>, Box<Error>> {
    let f = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(f);
    let mut rows: Vec<Vec<u8>> = Vec::new();

    for (idx, result) in rdr.deserialize().enumerate() {
        // the first row is always garbage (heading '----' markers)
        if idx == 0 {
            continue;
        }
        let record: AECFormalPreferencesRow = result?;
        for _i in record.preferences.split(",").map(|v| if v.len() == 0 { 0 } else if v == "*" || v == "/" { 1 } else { v.parse::<u8>().unwrap() }) {

        }
    }
    Ok((rows))
}