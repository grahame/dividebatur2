#[macro_use]
extern crate serde_derive;
extern crate rayon;
mod aec;

fn main() {
    let candidates = match aec::data::candidates::load("aec_data/fed2016/common/aec-senate-candidateinformation-20499.csv", "NT") {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read candidates file: {:?}", error);
        }
    };

    println!("{} candidates", candidates.len());

    let prefs = match aec::data::formalpreferences::load("aec_data/fed2016/wa/data/aec-senate-formalpreferences-20499-WA.csv") {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read formal preferences file: {:?}", error);
        }
    };

    println!("{} formal preferences", prefs.len());
}
