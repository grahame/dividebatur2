extern crate num;
extern crate rayon;
#[macro_use]
extern crate serde_derive;

mod aec;
mod defs;
mod engine;
mod senate2015;

fn main() {
    senate2015::run();
}
