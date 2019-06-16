use std::fs::File;
use engine::{CountEngine};
use configuration::{CountGroup, CountTask, Work};
use std::collections::{HashMap};
use defs::*;

#[derive(Serialize)]
struct Parameters {
    house: String,
    name: String,
    description: String,
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

#[derive(Serialize)]
struct Round {
    number: usize,
    note: String,
    elected: Vec<u32>,
}

#[derive(Serialize)]
struct ElectionSummary {

}

#[derive(Serialize)]
struct ExclusionSummary {

}

#[derive(Serialize)]
struct Summary {
    elected: Vec<ElectionSummary>,
    excluded: Vec<ExclusionSummary>,
}

#[derive(Serialize)]
struct Output {
    parameters: Option<Parameters>,
    candidates: Option<HashMap<usize, Candidate>>,
    parties: Option<HashMap<String, String>>,
    rounds: Vec<Round>,
    summary: Summary,
}

pub struct CountOutput {
    slug: String,
    output: Output,
}

pub trait CountOutputWriter {
    fn new(slug: &str) -> Self;
    fn set_parameters(&mut self, group: &CountGroup, task: &CountTask, engine: &CountEngine);
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
                rounds: Vec::new(),
                summary: Summary {
                    elected: Vec::new(),
                    excluded: Vec::new(),
                }
            },
        }
    }

    fn set_parameters(&mut self, group: &CountGroup, task: &CountTask, engine: &CountEngine) {
        self.output.parameters = Some(Parameters {
            total_papers: engine.total_papers,
            quota: engine.quota,
            house: group.house.clone(),
            vacancies: engine.vacancies,
            description: task.description.clone(),
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
struct OverallSummary {
    title: String,
    counts: HashMap<usize, CountSummary>,
}

pub fn write_summary(work: &Work) {
    let mut counts = HashMap::new();
    // FIXME: JSON output doesn't match multiple input file paradigm
    let group = work.groups.first().unwrap();
    for (idx, count) in group.counts.iter().enumerate() {
        counts.insert(idx, CountSummary {
            description: count.description.clone(),
            name: count.slug.clone(),
            path: count.slug.clone(),
            state: count.slug.clone()
        });
    }
    let summary = OverallSummary {
        title: group.description.clone(),
        counts: counts,
    };
    let output_file = format!("angular/data/count.json");
    let fd = File::create(output_file).unwrap();
    let result = serde_json::to_writer(fd, &summary);
    println!("{:?}", result);
}