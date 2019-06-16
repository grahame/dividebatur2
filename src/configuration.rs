use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Candidates {
    senate: String,
    all: String,
}

#[derive(Debug, Deserialize)]
struct Dataset {
    preferences: String,
}

#[derive(Debug, Deserialize)]
struct Tie {
    tie: Vec<String>,
    pick: String,
}

#[derive(Debug, Deserialize)]
struct Count {
    method: Option<String>,
    dataset: String,
    description: String,
    vacancies: usize,
    election_order_ties: Vec<Tie>,
    election_ties: Vec<Tie>,
    exclusion_ties: Vec<Tie>,
}

#[derive(Debug, Deserialize)]
struct Config {
    description: String,
    house: String,
    format: String,
    candidates: Candidates,
    dataset: HashMap<String, Dataset>,
    count: HashMap<String, Count>,
}

fn config_contents(input_file: &str) -> Result<Config, String> {
    let mut fd = match File::open(input_file) {
        Ok(fd) => fd,
        Err(e) => return Err(format!("unable to read {}: {}", input_file, e)),
    };

    let mut buf = String::new();
    fd.read_to_string(&mut buf).unwrap();

    let config: Config = match toml::from_str(&buf) {
        Ok(c) => c,
        Err(e) => return Err(format!("unable to parse {}: {}", input_file, e)),
    };

    Ok(config)
}

#[derive(Debug)]
pub struct CountTask {
    pub format: String,
    pub house: String,
    pub description: String,
    pub dataset: String,
    pub state: String,
    pub candidates: String,
    pub preferences: String,
    pub vacancies: usize,
    pub slug: String,
}

#[derive(Debug)]
pub struct Work {
    pub counts: Vec<CountTask>,
}

pub fn read_config(input_files: Vec<&str>) -> Work {
    let mut work = Work { counts: Vec::new() };
    for fname in input_files {
        let path = Path::new(fname);
        let dir = path.parent().unwrap().canonicalize().unwrap();
        let in_dir = |s: &str| -> String { dir.join(Path::new(s)).to_str().unwrap().to_string() };
        let config = match config_contents(fname) {
            Ok(c) => c,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        let mut new = config
            .count
            .iter()
            .map(|(slug, count)| {
                let dataset = match config.dataset.get(&count.dataset) {
                    Some(d) => d,
                    None => {
                        panic!();
                    }
                };
                CountTask {
                    overall_description: config.description.clone(),
                    state: slug.clone(),
                    slug: slug.clone(),
                    house: config.house.clone(),
                    description: count.description.clone(),
                    dataset: count.dataset.clone(),
                    candidates: in_dir(&config.candidates.all).clone(),
                    preferences: in_dir(&format!("{}/data/{}", slug, dataset.preferences)),
                    format: config.format.clone(),
                    vacancies: count.vacancies,
                }
            })
            .collect();
        work.counts.append(&mut new);
    }
    work
}