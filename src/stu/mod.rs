use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use tabled::{
    format::Format, locator::ByColumnName, object::Rows, object::*, style::Style, BorderText,
    Disable, Modify, Table, Tabled, Width,
};
use tempfile::Builder;

#[derive(Tabled, Serialize, Deserialize, Debug, Clone)]
pub struct Log {
    #[tabled(rename = "Subject")]
    pub subject: String,

    #[tabled(rename = "Topic")]
    pub topic: String,

    #[tabled(rename = "Date")]
    pub date: String,

    #[tabled(rename = "UID")]
    pub uid: String,

    #[tabled(rename = "Questions")]
    pub total_questions: usize,

    #[tabled(rename = "Right answers")]
    pub right_answers: usize,

    #[tabled(rename = "Percentage")]
    pub percentage: f32,
}
impl Log {
    pub fn new() -> Self {
        let random_uid: String = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
            .to_string();
        Self {
            subject: "unknown".to_string(),
            topic: "unknown".to_string(),
            date: "unknown".to_string(),
            uid: random_uid,
            total_questions: 0,
            right_answers: 0,
            percentage: 0.0,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Journal {
    pub name: String,
    pub logs: Vec<Log>,
}
impl Journal {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            logs: Vec::new(),
        }
    }
    pub fn add_log(&mut self, log: Log) {
        self.logs.push(log);
    }
}
pub fn get_journals(filepath: &str, journals: &mut Vec<Journal>) -> Result<(), ()> {
    let json_str: &str = &fs::read_to_string(filepath)
        .map_err(|err| eprintln!("{}: Could not read json filepath {err}", "ERROR".red()))?;

    let objects: Value = serde_json::from_str::<Value>(json_str).map_err(|err| {
        eprintln!(
            "{}: Could not create json object from string: {err}",
            "ERROR".red()
        )
    })?;

    for journal_objs in objects.as_array() {
        for journal_value in journal_objs {
            let name = journal_value["name"].as_str().ok_or_else(|| {
                eprintln!(
                    "{}: Value `Name` not found in {filepath} at `{value}`",
                    "ERROR".red(),
                    value = "Journals"
                );
            })?;
            let mut journal: Journal = Journal::new(name);
            if journal_value["logs"].is_null() {
                eprintln!(
                    "{}: Value `logs` not found in {filepath} at `{value}` journal",
                    "ERROR".red(),
                    value = journal_value["name"].as_str().unwrap()
                );
                return Err(());
            }

            for log_objs in journal_value["logs"].as_array() {
                for mut log_value in log_objs.clone() {
                    let percentage = utils::get_percentage(
                        *&log_value["right_answers"].as_u64().unwrap_or(0) as f32,
                        *&log_value["total_questions"].as_u64().unwrap_or(0) as f32,
                    );
                    log_value["percentage"] = serde_json::to_value(&percentage).unwrap();

                    let log: Log = serde_json::from_str(&log_value.to_string()).map_err(|err| {
                        eprintln!(
                            "{}: Could not deserialize json into log \
                             struct: {err}",
                            "ERROR".red()
                        );
                    })?;
                    journal.add_log(log.clone());
                }
            }
            journals.push(journal);
        }
    }

    Ok(())
}

pub fn show_metrics(journals: &Vec<Journal>) {
    for journal in journals {
        let mut sum_questions = 0;
        let mut sum_answers = 0;

        for log in journal.logs.iter() {
            sum_questions += log.total_questions;
            sum_answers += log.right_answers;
        }
        let mut sum_percentage = if sum_questions == 0 && sum_answers == 0 {
            "0.0".to_string()
        } else {
            utils::get_percentage(sum_answers as f32, sum_questions as f32).to_string()
        };

        sum_percentage.push('%');

        let sum_questions: &str = &sum_questions.to_string();
        let sum_answers: &str = &sum_answers.to_string();

        let mut builder = tabled::builder::Builder::default();
        builder.set_columns(["", "Total"]);
        builder.add_record(["Questions", sum_questions]);
        builder.add_record(["Answers", sum_answers]);
        builder.add_record(["Percentage", &sum_percentage]);
        let mut builder = builder.index();
        builder.hide_index();

        let mut metrics_table = builder.build();
        metrics_table
            .with(Width::list([10, 7]))
            .with(Style::rounded())
            .with(BorderText::new(0, format!("{}", journal.name)));

        println!("{metrics}", metrics = metrics_table.to_string());
    }
}

pub fn show_journals(journals: &mut Vec<Journal>) {
    for journal in journals.iter_mut() {
        unsafe {
            if crate::SORT {
                journal
                    .logs
                    .sort_by(|b, a| (a.percentage as i32).cmp(&(b.percentage as i32)));
            }
        }

        let mut table = Table::new(&journal.logs);
        table
            .with(
                Modify::new(ByColumnName::new("Percentage").not(Rows::first()))
                    .with(Format::new(|x| format!("{x}%"))),
            )
            .with(Style::rounded())
            .with(BorderText::new(0, format!("{name} ", name = journal.name)))
            .with(Modify::new(Rows::new(1..)).with(Width::truncate(15).suffix("...")))
            .with(Width::justify(15));

        println!("{table}", table = table.to_string());
    }
}

pub fn show_log(log: &Log) {
    let table = Table::new(vec![log])
        .with(Disable::column(ByColumnName::new("Subject")))
        .with(Style::rounded())
        .with(BorderText::new(0, format!("{}", log.subject)))
        .to_string();

    println!("{table}");
}

fn log_from_tf(buf: String) -> Result<Log, ()> {
    let mut lines = buf.lines().enumerate().peekable();
    let mut log: Log = Log::new();

    while let Some(current) = lines.next() {
        if let Some(&next) = lines.peek() {
            let next_line = next.1;
            let line_number = next.0 + 1;

            let mut quit = false;

            match current.1.trim() {
                "[type here]" => quit = true,
                "Subject" => log.subject = utils::remove_brackets(next_line),
                "Topic" => log.topic = utils::remove_brackets(next_line),

                "Total Questions" => {
                    log.total_questions =
                        utils::remove_brackets(next_line).parse().map_err(|err| {
                            eprintln!(
                                "{}: Failed to read log file: {err} {next_line} at line {}",
                                "ERROR".red(),
                                line_number
                            )
                        })?
                }
                "Right Answers" => {
                    log.right_answers =
                        utils::remove_brackets(next_line).parse().map_err(|err| {
                            eprintln!(
                                "{}: Failed to read log file: {err} {next_line} at line {}",
                                "ERROR".red(),
                                line_number
                            )
                        })?
                }
                _ => (),
            }
            if quit == true {
                eprintln!(
                    "{text}",
                    text = "a field was left unchanged, log was not added".red()
                );
                return Err(());
            }
        }
    }

    log.percentage = utils::get_percentage(log.right_answers as f32, log.total_questions as f32);

    Ok(log)
}

pub fn make_log(name: &str) -> Result<Log, ()> {
    let mut tf = Builder::new()
        .prefix("stu-log_")
        .suffix(".txt")
        .rand_bytes(4)
        .tempfile()
        .map_err(|err| {
            eprintln!("{}: Could not create tempfile: {err}", "ERROR".red());
        })?;

    let date = utils::get_date();
    let note_builder_text: &str = &format!(
        "\
        STU Note Builder\n\
        Journal: {name}\n\
        Date: {date}\n\n\
        \
        ------------------\n\n\
        **TYPE INSIDE BRACKETS**\n\
        *edit, save and exit*\n\
        *to cancel just leave some field unchanged*\n\n\
        \
        Subject\n\
        [type here]\n\n\
        \
        Topic\n\
        [type here]\n\n\
        \
        Total Questions\n\
        [type here]\n\n\
        \
        Right Answers\n\
        [type here]\n\
        "
    );

    write!(tf, "{}", &note_builder_text).unwrap();
    tf.flush().unwrap();

    utils::edit_text(tf.path().display().to_string())?;

    tf.flush().unwrap();
    tf.rewind().unwrap();

    let mut buf = String::new();
    tf.read_to_string(&mut buf).unwrap();

    let mut log: Log = log_from_tf(buf)?;
    log.date = date;

    tf.close().map_err(|err| {
        eprintln!("{}: Could not delete temporary file: {err}", "ERROR".red());
    })?;

    Ok(log)
}

pub fn list_journals(journals: &Vec<Journal>) {
    let buf = format!(
        "{} {} {}",
        "There's".bold(),
        journals.len().to_string().red().bold(),
        "Journals in database".bold()
    );
    println!("{buf}\n");
    for journal in journals.iter() {
        println!("- {name}", name = journal.name.bold());
        println!("    Logs");
        for logs in journal.logs.iter() {
            println!("       uid: {id} [{date}]", id = logs.uid, date = logs.date);
        }
    }
    println!();
}

pub fn sync_data(journals: String, filepath: &str) -> Result<(), ()> {
    let mut file = File::create(filepath).map_err(|err| {
        eprintln!("{}: Could not create file: {err}", "ERROR".red());
    })?;

    write!(file, "{}", journals).unwrap();

    file.sync_all().map_err(|err| {
        eprintln!("{}: Could not sync OS data: {err}", "ERROR".red());
    })?;

    Ok(())
}

pub fn query_for(str: &str, filepath: &str) -> Result<(), ()> {
    let mut journals: Vec<Journal> = Vec::new();
    get_journals(filepath, &mut journals)?;

    let mut query_journal: Journal = Journal::new("Query");
    for journal in journals {
        if journal.name.to_lowercase() == str {
            show_journals(&mut vec![journal]);
            return Ok(());
        }
        for log in journal.logs.into_iter() {
            if str == log.subject.to_lowercase()
                || str == log.topic.to_lowercase()
                || str == log.date.to_lowercase()
            {
                query_journal.add_log(log);
            }
        }
    }
    if query_journal.logs.len() > 0 {
        show_journals(&mut vec![query_journal]);
        return Ok(());
    }

    eprintln!("{}", format!("unsuccessfully <{str}> query").red());
    Err(())
}

pub fn query_uid(uid: &str, filepath: &str) -> Result<(), ()> {
    let mut journals: Vec<Journal> = Vec::new();
    get_journals(filepath, &mut journals)?;

    let mut log: Option<Log> = None;
    for journal in journals.iter() {
        log = journal.logs.iter().cloned().filter(|x| x.uid == uid).next();
        if log.is_some() {
            break;
        }
    }

    if log.is_none() {
        eprintln!("{}", format!("log with <{uid}> UID not found").red());
        return Err(());
    }

    show_log(&log.unwrap());
    return Ok(());
}

pub fn edit_log(log: Log) -> Result::<Log, ()> {
    let mut tf = Builder::new()
        .prefix("stu-log_")
        .suffix(".txt")
        .rand_bytes(4)
        .tempfile()
        .map_err(|err| {
            eprintln!("{}: Could not create tempfile: {err}", "ERROR".red());
        })?;

    let note_builder_text: &str = &format!(
        "\
        STU Note edit\n\n\
        ------------------\n\n\
        **TYPE INSIDE BRACKETS**\n\
        *edit, save and exit*\n\
        *to cancel just leave some field unchanged*\n\n\
        \
        Subject\n\
        [{subject}]\n\n\
        \
        Topic\n\
        [{topic}]\n\n\
        \
        Total Questions\n\
        [{questions}]\n\n\
        \
        Right Answers\n\
        [{answers}]\n\
        ",
        subject   = log.subject,
        topic     = log.topic,
        questions = log.total_questions,
        answers   = log.right_answers
    );

    write!(tf, "{}", &note_builder_text).unwrap();
    tf.flush().unwrap();

    utils::edit_text(tf.path().display().to_string())?;

    tf.flush().unwrap();
    tf.rewind().unwrap();

    let mut buf = String::new();
    tf.read_to_string(&mut buf).unwrap();

    let mut new_log: Log = log_from_tf(buf)?;

    new_log.uid = log.uid;
    new_log.date = log.date;

    tf.close().map_err(|err| {
        eprintln!("{}: Could not delete temporary file: {err}", "ERROR".red());
    })?;

    Ok(new_log)
}

pub mod utils;