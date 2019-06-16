use std::fs::File;
use engine::{CountEngine};
use configuration::{CountTask, Work};
use std::collections::{HashMap};
use defs::*;

#[derive(Serialize)]
struct Parameters {
    name: String,
    description: String,
    house: String,
    state: String,
    quota: u32,
    vacancies: u32,
    total_papers: u32,
}

#[derive(Serialize)]
struct Candidate {
    id: usize,
    title: String,
    party: String,
}

struct Round {
    number: usize,
    note: String,
    elected: Vec<CandidateIndex>,
}

#[derive(Serialize)]
struct Output {
    parameters: Option<Parameters>,
    candidates: Option<HashMap<usize, Candidate>>,
    parties: Option<HashMap<String, String>>,
}

pub struct CountOutput {
    slug: String,
    output: Output,
}

pub trait CountOutputWriter {
    fn new(slug: &str) -> Self;
    fn set_parameters(&mut self, task: &CountTask, engine: &CountEngine);
    fn set_candidates(&mut self, cd: &CandidateData);
    fn close(&self);
}

impl CountOutputWriter for CountOutput {
    fn new(slug: &str) -> CountOutput {
        CountOutput {
            slug: slug.to_string(),
            output: Output {
                parameters: None,
                candidates: None,
                parties: None,
            },
        }
    }

    fn set_parameters(&mut self, task: &CountTask, engine: &CountEngine) {
        self.output.parameters = Some(Parameters {
            total_papers: engine.total_papers,
            quota: engine.quota,
            vacancies: engine.vacancies,
            description: task.description.clone(),
            house: task.house.clone(),
            name: self.slug.clone(),
            state: task.dataset.clone(),
        });
    }

    fn set_candidates(&mut self, cd: &CandidateData) {
        let mut p = HashMap::new();
        let mut c = HashMap::new();
        for (idx, (name, party)) in cd.names.iter().zip(cd.parties.iter()).enumerate() {
            c.insert(idx, Candidate {
                id: idx,
                title: name.clone(),
                party: party.clone(),
            });
            p.insert(party.clone(), party.clone());
        }
        self.output.candidates = Some(c);
        self.output.parties = Some(p);
    }

    fn close(&self) {
        let output_file = format!("angular/data/{}.json", self.slug);
        let fd = File::create(output_file).unwrap();
        let result = serde_json::to_writer(fd, &self.output);
        println!("{:?}", result);
    }
}

#[derive(Serialize)]
struct CountSummary {
    description: String,
    name: String,
    path: String,
    state: String,
}


#[derive(Serialize)]
struct Summary {
    title: String,
    counts: HashMap<usize, CountSummary>,
}

pub fn write_summary(work: &Work) {
    let counts = HashMap::new();
    for (idx, count) in work.counts.iter().enumerate() {
    }
    let summary = Summary {
        title: work.,
        counts: counts,
    };
}