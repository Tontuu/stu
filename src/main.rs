use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::io::{Read, Seek, Write};
use std::process::{Command, ExitCode};
use std::result::Result;
use std::time::{SystemTime, UNIX_EPOCH};
use tabled::{style::Style, BorderText, Table, Tabled, Width};
use tempfile::Builder;

static DEFAULT_EDITOR: &str = "vim";

#[derive(Tabled, Serialize, Deserialize, Debug, Clone)]
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

    uid: String,
}

impl Log {
    fn new(
        subject: &str,
        topic: &str,
        date: &str,
        total_questions: usize,
        right_answers: usize,
    ) -> Self {
        let random_uid: String = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
            .to_string();

        Self {
            subject: subject.to_string(),
            topic: topic.to_string(),
            date: date.to_string(),
            uid: random_uid,
            total_questions,
            right_answers,
            percentage: ((right_answers * 100) as f32 / total_questions as f32).to_string(),
        }
    }
    fn default() -> Self {
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
            percentage: "unknown".to_string(),
        }
    }

    fn get_percentage(&mut self) {
        self.percentage =
            ((self.right_answers * 100) as f32 / self.total_questions as f32).to_string();
        self.percentage.push('%');
    }

    fn get_date(&mut self) {
        let date_process = Command::new("/usr/bin/date")
            .arg("+%m/%d/%Y %r")
            .output()
            .expect("ERROR: Could not run date process");

        let output = std::str::from_utf8(&date_process.stdout)
            .unwrap_or_else(|_| "unknown")
            .trim();
        self.date = output.to_string();
    }
}

#[derive(Debug)]
struct Journal {
    name: String,
    logs: Vec<Log>,
}

impl Journal {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            logs: Vec::new(),
        }
    }

    fn add_log(&mut self, log: Log) {
        self.logs.push(log);
    }
}
fn usage(program: &str) {
    eprintln!(
        "{usage}: {program} [SUBCOMMAND] [OPTIONS]\n",
        usage = "Usage".red()
    );
    eprintln!("{subcommands}:", subcommands = "Subcommands".red());
    eprintln!("    show                   print all user journals");
    eprintln!("    add     <journal>      add record log into journal");
    eprintln!("    add -j  <value>        add new journal with the given <value> name");
    eprintln!("    get      <query>       search for <query> and print results");
    eprintln!("                ╰------>   query can be: [journal, subject, topic, \"MM/DD/YYYY\"");
    eprintln!()
}

fn get_journals(filepath: &str, journals: &mut Vec<Journal>) -> Result<(), ()> {
    let json_str: &str = &fs::read_to_string(filepath)
        .map_err(|err| eprintln!("{}: Could not read json filepath {err}", "ERROR".red()))?;

    let objects: Value = serde_json::from_str::<Value>(json_str).map_err(|err| {
        eprintln!(
            "{}: Could not create json object from string: {err}",
            "ERROR".red()
        )
    })?;

    for journal_objs in objects["Journals"].as_array() {
        for journal_value in journal_objs {
            let name = journal_value["Name"].as_str().ok_or_else(|| {
                eprintln!(
                    "{}: Value `Name` not found in {filepath} at `{value}`",
                    "ERROR".red(),
                    value = "Journals"
                );
            })?;
            let mut journal: Journal = Journal::new(name);
            if journal_value["Logs"].is_null() {
                eprintln!(
                    "{}: Value `Logs` not found in {filepath} at `{value}` journal",
                    "ERROR".red(),
                    value = journal_value["Name"].as_str().unwrap()
                );
                return Err(());
            }

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

fn show(journals: &Vec<Journal>) {
    for journal in journals {
        let mut table = Table::new(&journal.logs);
        table
            .with(Style::rounded())
            .with(BorderText::new(0, format!("{name} ", name = journal.name)))
            .with(Width::justify(20));

        println!("{table}", table = table.to_string())
    }
}

fn edit_text(filepath: String) -> Result<(), ()> {
    // TODO: Give user the external option of choosing the EDITOR
    let env_editor: String = env::var("EDITOR").unwrap_or_else(|_| DEFAULT_EDITOR.to_string());

    let mut editor_process = Command::new(&env_editor)
        .arg(filepath)
        .spawn()
        .expect(format!("ERROR: Could not start {} editor", env_editor).as_str());

    let _exit_code = editor_process.wait().map_err(|err| {
        eprintln!("{}: {err}", "ERROR".red());
    })?;

    Ok(())
}

fn remove_brackets(string: &str) -> String {
    string
        .chars()
        .filter(|c| c != &'[' && c != &']')
        .collect::<String>()
}

fn log_from_tf(buf: String) -> Result<Log, ()> {
    let mut lines = buf.lines().enumerate().peekable();
    let mut log: Log = Log::default();

    while let Some(current) = lines.next() {
        if let Some(&next) = lines.peek() {
            let next_line = next.1;
            let line_number = next.0 + 1;

            let mut quit = false;

            match current.1.trim() {
                "[type here]" => quit = true,
                "Subject" => log.subject = remove_brackets(next_line),
                "Topic" => log.topic = remove_brackets(next_line),

                "Total Questions" => {
                    log.total_questions = remove_brackets(next_line).parse().map_err(|err| {
                        eprintln!(
                            "{}: Failed to read log file: {err} {next_line} at line {}",
                            "ERROR".red(),
                            line_number
                        )
                    })?
                }
                "Right Answers" => {
                    log.right_answers = remove_brackets(next_line).parse().map_err(|err| {
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

    log.get_percentage();
    log.get_date();

    Ok(log)
}

fn make_log(name: &str) -> Result<Log, ()> {
    let mut tf = Builder::new()
        .prefix("stu-log_")
        .suffix(".md")
        .rand_bytes(4)
        .tempfile()
        .map_err(|err| {
            eprintln!("{}: Could not create tempfile: {err}", "ERROR".red());
        })?;

    let note_builder_text: &str = &format!(
        "\
           STU Note Builder\n\
           Journal: {name}\n\n\
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

    edit_text(tf.path().display().to_string())?;

    tf.flush().unwrap();
    tf.rewind().unwrap();

    tf.flush().unwrap();
    tf.rewind().unwrap();

    let mut buf = String::new();
    tf.read_to_string(&mut buf).unwrap();

    let log: Log = log_from_tf(buf)?;

    tf.close().map_err(|err| {
        eprintln!("{}: Could not delete temporary file: {err}", "ERROR".red());
    })?;

    Ok(log)
}

fn list_journals(journals: &Vec<Journal>) {
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

fn setup() -> Result<(), ()> {
    let mut args = env::args();
    let program = args.next().expect("path to program is provided");

    let subcommand = args.next().ok_or_else(|| {
        usage(&program);
        eprintln!("{}: Subcommand is needed", "ERROR".red());
    })?;

    match subcommand.as_str() {
        "show" => {
            let mut journals: Vec<Journal> = Vec::new();
            get_journals("../data.json", &mut journals)?;

            if journals.len() == 0 {
                usage(&program);
                eprintln!(
                    "{text}",
                    text = "There's no journals at the moment, create one with\
                           the command `stu -j add <name>`".red()
                );
                return Err(());
            }

            show(&journals);
        }
        "get" => {
            unimplemented!();
        }
        "add" => {
            match args.next().as_deref() {
                Some("-j") => {
                    if args.next().is_none() {
                        eprintln!("{}: New journal name was not provided", "ERROR".red());
                        return Err(());
                    }
                    println!("Adding new journal");
                }
                Some(user_journal_query) => {
                    let mut journals: Vec<Journal> = Vec::new();
                    get_journals("../data.json", &mut journals)?;
                    let result = journals
                        .iter()
                        .filter(|x| x.name == user_journal_query)
                        .next();
                    match result {
                        None => {
                            list_journals(&journals);
                            let text = format!(
                                r"{text1}{name}{text2} {prompt}",
                                text1 = "Journal with the name `".red(),
                                name = user_journal_query.red(),
                                text2 = "` was not found, do you \
                                           want to create one? ".red(),
                                prompt = "[y/n]"
                            );
                            println!("{text}");
                        }

                        Some(_) => {
                            let new_log = make_log(user_journal_query)?;
                            // journal.push(new_log);
                            for journal in journals.iter_mut() {
                                if journal.name == user_journal_query {
                                    journal.add_log(new_log.clone());
                                }
                            }
                            // TODO[1]: Update json file 
                            println!("Name {:#?}", user_journal_query);
                            println!("{:#?}", journals)
                        }
                    }
                }
                None => {
                    eprintln!("{}: Journal name to query was not provided", "ERROR".red());
                    return Err(());
                }
            }
        }
        _ => {
            usage(&program);
            eprintln!("{}: Unexpected subcommand: {subcommand}", "ERROR".red());
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

// TODO: Implement query log
// TODO: Implement add log to journal
// TODO: Implement create journal
