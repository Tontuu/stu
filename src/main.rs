use tabled::{Table, Tabled};
use std::fs;
use serde_json::{Result, Value};
use serde::{Serialize, Deserialize};

extern crate quick_csv;

#[derive(Tabled)]
#[tabled(rename_all = "CamelCase")]
#[derive(Serialize, Deserialize, Debug)]
struct Log {
    #[tabled(rename = "Subject")]
    subject: String,

    #[tabled(rename = "Topic")]
    topic: String,

    #[tabled(rename = "Date")]
    date: String,

    #[tabled(rename = "Total answers")]
    total_questions: usize,

    #[tabled(rename = "Right answers")]
    right_answers: usize,

    #[tabled(rename = "Percentage")]
    percentage: String,
}

impl Log {
    fn calculate_percentage(&self) -> f32 {
        (self.right_answers * 100) as f32 / self.total_questions as f32
    }
}

#[derive(Debug)]
struct Journal {
    exam: String,
    logs: Vec<Log>,
}

impl Journal {
    fn new(exam: String , logs: Vec<Log>) -> Self {
        // let mut percentage = (right_answers*100) as f32 / total_questions as f32;
        Self {
            exam,
            logs,
        }
    }
}

fn main() -> Result<()> {
    let mut journals: Vec<Journal> = Vec::new();
    
    let json_str: &str = &fs::read_to_string("data.json").expect("ERROR: Could not read json file");
    let objects: Value  = serde_json::from_str::<Value>(json_str)
        .expect("ERROR: Could not make json object from string");

    for journal_objs in objects["Journals"].as_array() {
        for journal_value in journal_objs {
            let mut logs: Vec<Log> = Vec::new();

            for log_objs in journal_value["Logs"].as_array() {
                for mut log_value in log_objs.clone() {
                    let percentage = format!("{}%", (&log_value["right_answers"]
                        .as_u64()
                        .expect("ERROR: Could not extract number from value")
                        * 100 /
                        &log_value["total_questions"]
                        .as_u64()
                        .expect("ERROR: Could not extract number from value")).to_string());
                    log_value["percentage"] = serde_json::to_value(&percentage).unwrap();

                    let log: Log = serde_json::from_str(&log_value.to_string())?;
                    logs.push(log);
                }
            }
            let exam = journal_value["Exam"].as_str().unwrap().to_string();
            journals.push(Journal::new(exam, logs));
        }
    }
    for journal in journals {
        let table = Table::new(journal.logs).to_string();
        println!("{table}")
    }
    Ok(())
}
