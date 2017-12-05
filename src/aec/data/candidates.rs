//
// Parse the senate candidate information CSV file.
// Example file: http://results.aec.gov.au/20499/website/External/aec-senate-candidateinformation-20499.zip
//

extern crate csv;

use std::error::Error;
use std::fs::File;

#[derive(Debug,Deserialize)]
pub struct AECAllCandidateRow {
    // the bits we actually care about
    pub ticket: String,
    pub ballot_position: u32,
    pub surname: String,
    pub ballot_given_nm: String,
    pub party_ballot_nm: String,
    // ... and the other bits
    txn_nm: String,
    nom_ty: String,
    state_ab: String,
    div_nm: String,
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

pub fn load(filename: &str, state: &str) -> Result<Vec<AECAllCandidateRow>, Box<Error>> {
    let f = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(f);
    let mut rows: Vec<AECAllCandidateRow> = Vec::new();
    for result in rdr.deserialize() {
        let record: AECAllCandidateRow = result?;
        // we only want senate nominations
        if record.nom_ty != "S" {
            continue;
        }
        if record.state_ab != state {
            continue;
        }
        rows.push(record);
    }
    // a little subtle: we need to sort in ballot paper order, which means ticket ordering:
    //   A, B, C, ... Z, AA, AB, .. AZ, etc
    // (rather than a standard lexical sort)
    rows.sort_by(|a, b| (a.ticket.len(), &a.ticket, &a.ballot_position).cmp(&(b.ticket.len(), &b.ticket, &b.ballot_position)));
    Ok((rows))
}