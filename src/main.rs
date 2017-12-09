#[macro_use]
extern crate serde_derive;
extern crate rayon;

mod aec;
mod defs;
mod senate2015;

fn main() {
    senate2015::run();
}