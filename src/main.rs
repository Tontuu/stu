use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::process::ExitCode;
use std::result::Result;
use tabled::{style::Style, BorderText, Table, Tabled, Width};

extern crate quick_csv;

#[derive(Tabled, Serialize, Deserialize, Debug)]
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
    fn new(
        subject: &str,
        topic: &str,
        date: &str,
        total_questions: usize,
        right_answers: usize,
    ) -> Self {
        Self {
            subject: subject.to_string(),
            topic: topic.to_string(),
            date: date.to_string(),
            total_questions,
            right_answers,
            percentage: ((right_answers * 100) as f32 / total_questions as f32).to_string(),
        }
    }
}

#[derive(Debug)]
struct Journal {
    exam: String,
    logs: Vec<Log>,
}

impl Journal {
    fn new(exam: &str) -> Self {
        Self {
            exam: exam.to_string(),
            logs: Vec::new(),
        }
    }

    fn add_log(&mut self, log: Log) {
        self.logs.push(log);
    }
}
fn usage(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands:");
    eprintln!("    show              print all user journals");
    eprintln!("    add <journal>     interactively add log to journal");
    eprintln!("    get <query>       search for <query> and print results");
    eprintln!("       -> query can be: [journal, subject, topic, \"DD/MM/YYYY\"");
}

fn get_journals(filepath: &str, journals: &mut Vec<Journal>) -> Result<(), ()> {
    let json_str: &str = &fs::read_to_string(filepath).expect("ERROR: Could not read json file");
    let objects: Value = serde_json::from_str::<Value>(json_str)
        .expect("ERROR: Could not make json object from string");

    for journal_objs in objects["Journals"].as_array() {
        for journal_value in journal_objs {
            let exam = journal_value["Exam"].as_str().unwrap();
            let mut journal: Journal = Journal::new(exam);

            for log_objs in journal_value["Logs"].as_array() {
                for mut log_value in log_objs.clone() {
                    let percentage = format!(
                        "{}%",
                        (&log_value["right_answers"]
                            .as_u64()
                            .expect("ERROR: Could not extract number from value")
                            * 100
                            / &log_value["total_questions"]
                                .as_u64()
                                .expect("ERROR: Could not extract number from value"))
                            .to_string()
                    );
                    log_value["percentage"] = serde_json::to_value(&percentage).unwrap();

                    let log: Log = serde_json::from_str(&log_value.to_string()).map_err(|err| {
                        eprintln!("ERROR: Could not deserialize json into log struct: {err}");
                    })?;
                    journal.add_log(log);
                }
            }
            journals.push(journal);
        }
    }

    Ok(())
}

fn show(journals: &Vec<Journal>) {
    for journal in journals {
        let mut table = Table::new(&journal.logs);
        table
            .with(Style::rounded())
            .with(BorderText::new(0, format!("{exam} ", exam = journal.exam)))
            .with(Width::justify(20));

        println!("{table}", table = table.to_string())
    }
}

fn setup() -> Result<(), ()> {
    let mut args = env::args();
    let program = args.next().expect("path to program is provided");

    let subcommand = args.next().ok_or_else(|| {
        usage(&program);
        eprintln!("ERROR: Subcommand is needed");
    })?;

    let mut journals: Vec<Journal> = Vec::new();
    get_journals("data.json", &mut journals)?;

    match subcommand.as_str() {
        "show" => {
            show(&journals);
        }
        "get" => {
            unimplemented!();
        }
        "add" => {
            unimplemented!();
        }
        _ => {
            usage(&program);
            eprintln!("ERROR: Unexpected subcommand: {subcommand}");
            return Err(());
        }
    }

    Ok(())
}
fn main() -> ExitCode {
    match setup() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}
