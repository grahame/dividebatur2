#[macro_use]
extern crate serde_derive;
extern crate rayon;
extern crate num;

mod aec;
mod engine;
mod defs;
mod senate2015;

fn main() {
    senate2015::run();
}