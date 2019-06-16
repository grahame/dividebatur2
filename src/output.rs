use std::fs::File;

pub struct CountOutput {
    slug: String,
    fd: File
}

pub trait CountOutputWriter {
    fn new(slug: &str) -> Self;
}

impl CountOutputWriter for CountOutput {
    fn new(slug: &str) -> CountOutput {
        let output_file = format!("output/{}.json", slug);
        // serde_json::to_writer(fd, cd);

        CountOutput {
            slug: slug.to_string(),
            fd: File::create(output_file).unwrap(),
        }
    }
}

