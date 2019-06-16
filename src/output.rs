use std::fs::File;
use engine::{CountEngine};
use configuration::{CountTask};

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
struct Output {
    parameters: Option<Parameters>,
}

pub struct CountOutput {
    slug: String,
    output: Output,
}

pub trait CountOutputWriter {
    fn new(slug: &str) -> Self;
    fn set_parameters(&mut self, task: &CountTask, engine: &CountEngine);
    fn close(&self);
}

impl CountOutputWriter for CountOutput {
    fn new(slug: &str) -> CountOutput {
        CountOutput {
            slug: slug.to_string(),
            output: Output {
                parameters: None,
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

    fn close(&self) {
        let output_file = format!("output/{}.json", self.slug);
        let fd = File::create(output_file).unwrap();
        let result = serde_json::to_writer(fd, &self.output);
        println!("{:?}", result);
    }
}
