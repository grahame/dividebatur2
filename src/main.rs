#[macro_use]
extern crate serde_derive;

mod aec_candidates; 

fn main() {
    println!("Hello, world!");
    let candidates = match aec_candidates::load_aec_candidates("aec_data/fed2016/common/2016federalelection-all-candidates-nat-30-06-924.csv".to_owned()) {
        Ok(rows) => rows,
        Err(error) => {
            panic!("Couldn't read candidates file: {:?}", error);
        }
    };
    println!("{} candidates", candidates.len());
}
