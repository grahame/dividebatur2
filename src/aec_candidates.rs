
extern crate csv;

use std::error::Error;
use std::fs::File;

#[derive(Debug,Deserialize)]
pub struct AECAllCandidateRow {
    txn_nm: String,
    nom_ty: String,
    state_ab: String,
    div_nm: String,
    ticket: String,
    ballot_position: String,
    surname: String,
    ballot_given_nm: String,
    party_ballot_nm: String,
    occupation: String,
    address_1: String,
    address_2: String,
    postcode: String,
    suburb: String,
    address_state_ab: String,
    contact_work_ph: String,
    contact_home_ph: String,
    postal_address_1: String,
    postal_address_2: String,
    postal_suburb: String,
    postal_postcode: String,
    contact_fax: String,
    postal_state_ab: String,
    contact_mobile_no: String,
    contact_email: String,
}

pub fn load_aec_candidates(filename: String) -> Result<Vec<AECAllCandidateRow>, Box<Error>> {
    let f = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(f);
    let mut rows: Vec<AECAllCandidateRow> = Vec::new();
    for result in rdr.deserialize() {
        let record: AECAllCandidateRow = result?;
        rows.push(record);
    }
    Ok((rows))
}