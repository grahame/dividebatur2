#[macro_use]
extern crate serde_derive;

mod aec;

fn main() {
    println!("Hello, world!");
    let candidates = match aec::data::candidates::load_aec_candidates("aec_data/fed2016/common/aec-senate-candidateinformation-20499.csv", "NSW") {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read candidates file: {:?}", error);
        }
    };
    println!("{} candidates", candidates.len());
}
