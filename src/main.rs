use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::io::{Read, Seek, Write};
use std::fs::File;
use std::process::{Command, ExitCode};
use std::result::Result;
use std::time::{SystemTime, UNIX_EPOCH};
use tabled::{style::Style, BorderText, Table, Tabled, Width, Modify, object::Rows,
             Disable, locator::ByColumnName, format::Format, object::*};
use tempfile::Builder;

static DEFAULT_EDITOR: &str = "vim";
static mut SORT: bool = false;

#[derive(Tabled, Serialize, Deserialize, Debug, Clone)]
struct Log {
    #[tabled(rename = "Subject")]
    subject: String,

    #[tabled(rename = "Topic")]
    topic: String,

    #[tabled(rename = "Date")]
    date: String,

    #[tabled(rename = "UID")]
    uid: String,

    #[tabled(rename = "Questions")]
    total_questions: usize,

    #[tabled(rename = "Right answers")]
    right_answers: usize,

    #[tabled(rename = "Percentage")]
    percentage: f32,
}

impl Log {
    fn new() -> Self {
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
struct Journal {
    name: String,
    logs: Vec<Log>,
}


fn get_date() -> String {
    let date_process = Command::new("/usr/bin/date")
        .arg("+%m/%d/%Y")
        .output()
        .expect("ERROR: Could not run date process");

    let output = std::str::from_utf8(&date_process.stdout)
        .unwrap_or_else(|_| "unknown")
        .trim();
    return output.to_string();
}

fn get_percentage(amount: f32, total: f32) -> f32 {
    let result = (amount * 100.0) / total;
    let rounded = result.round();
    return rounded;
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
    println!(
        "{usage}: {program} <subcommand> <options>\n",
        usage = "Usage".red()
    );
    println!("{subcommands}:", subcommands = "Subcommands".red());
    println!("    -h      --help                    print help");
    println!();
    println!("    show   <subcommand>               print all user journals, use -m if you wanna print the metrics");
    println!("                ╰------------------------> print metrics: \"-m\"");
    println!();
    println!("    remove <subcommand> <value>       remove a log with the given <value>");
    println!("                │          ╰-------------> value can be: [UID, journal]");
    println!("                ╰------------------------> remove journal: \"-j\"");
    println!();
    println!("    add    <subcommand> <value>       add either a new log or journal");
    println!("                ╰------------------------> add journal: \"-j\"");
    println!();
    println!("    get    <subcommand> <query>       search for <query> and print results");
    println!("                │          ╰-------------> query can be: [UID, journal, subject, topic, \"MM/DD/YYYY\"]");
    println!("                ╰------------------------> sort query: \"-s\"");
    println!()
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
                    let percentage = get_percentage(
                        *&log_value["right_answers"].as_u64().unwrap_or(0) as f32,
                        *&log_value["total_questions"].as_u64().unwrap_or(0) as f32
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

fn show_metrics(journals: &Vec<Journal>) {
    for journal in journals {
        let mut sum_questions = 0;
        let mut sum_answers = 0;

        for log in journal.logs.iter() {
            sum_questions += log.total_questions;
            sum_answers += log.right_answers;
        }
        let sum_percentage: &str = &get_percentage(sum_answers as f32, sum_questions as f32).to_string();
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

fn show_journals(journals: &mut Vec<Journal>) {
    for journal in journals.iter_mut() {
        unsafe {
            if SORT {
                journal.logs.sort_by(|b, a| (a.percentage as i32).cmp(&(b.percentage as i32)));
                // journal.logs.sort_by_key(|x| { x.percentage.pop(); x.percentage.parse::<i32>().unwrap() });
            }
        }

        let mut table = Table::new(&journal.logs);
        table
            .with(Modify::new(ByColumnName::new("Percentage").not(Rows::first())).with(Format::new(|x| format!("{x}%"))))
            .with(Style::rounded())
            .with(BorderText::new(0, format!("{name} ", name = journal.name)))
            .with(Modify::new(Rows::new(1..)).with(Width::truncate(15).suffix("...")))
            .with(Width::justify(15));

        println!("{table}", table = table.to_string());
    }
}

fn show_log(log: &Log) {
    let table = Table::new(vec![log])
        .with(Disable::column(ByColumnName::new("Subject")))
        .with(Style::rounded())
        .with(BorderText::new(0, format!("{}", log.subject)))
        .to_string();

    println!("{table}");
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
    let mut log: Log = Log::new();

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

    log.percentage = get_percentage(log.right_answers as f32, log.total_questions as f32);

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

    let date = get_date();
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

    edit_text(tf.path().display().to_string())?;

    tf.flush().unwrap();
    tf.rewind().unwrap();

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

fn update_json(journals: String, filepath: &str) -> Result<(), ()>{
    let mut file = File::create(filepath).map_err(|err| {
        eprintln!("{}: Could not create file: {err}", "ERROR".red());
    })?;

    write!(file, "{}", journals).unwrap();
    
    file.sync_all().map_err(|err| {
        eprintln!("{}: Could not sync OS data: {err}", "ERROR".red());
    })?;

    Ok(())
}

fn is_string_numeric(str: &str) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }

    return true;
}

fn is_string_alphanumeric(str: &str) -> bool {
    for c in str.chars() {
        if c != '/' && !c.is_alphanumeric() && !c.is_whitespace() {
            return false;
        }
    }
    return true;
}

fn query_for(str: &str, filepath: &str) -> Result<(), ()> {
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

fn query_uid(uid: &str, filepath: &str) -> Result<(), ()> {
    if uid.len() != 9 {
        eprintln!("{}: <{uid}> is an invalid UID", "ERROR".red());
        return Err(());
    }

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

fn setup() -> Result<(), ()> {
    let filepath: &str = "foo.json";
    let mut args = env::args();
    let program = args.next().expect("path to program is provided");

    let subcommand = args.next().ok_or_else(|| {
        usage(&program);
        eprintln!("{}: Subcommand is needed", "ERROR".red());
    })?;

    match subcommand.as_str() {
        "-h" | "--help" => {
            usage(&program);
            return Ok(());
        }
        "show" => {
            let mut journals: Vec<Journal> = Vec::new();
            get_journals(filepath, &mut journals)?;

            if journals.len() == 0 {
                usage(&program);
                eprintln!("{}", format!("There's no journals at the moment, create one with\
                            the command `stu -j add <name>`").red());

                return Err(());
            }

            match args.next().as_deref() {
                Some("-m") => { show_metrics(&journals);                          return Ok(()); }
                None       => { show_journals(&mut journals);                     return Ok(()); }
                Some(_)    => { eprintln!("{}: Unknown argument", "ERROR".red()); return Err(());}
            }


        }
        "get" => {
            let value = args.next();
            match value {
                Some(mut str) => {
                    if is_string_numeric(&str) {
                        return query_uid(&str, filepath);
                    }

                    if str == "-s" {
                        match args.next() {
                            Some(new_str) => unsafe { SORT = true; str = new_str; },
                            None => { eprintln!("ERROR: Unknown argument"); return Err(()) },
                        }
                    }

                    if is_string_alphanumeric(&str) {
                        return query_for(&str.to_lowercase(), filepath);
                    }

                    eprintln!("ERROR: Unknown query type");
                    return Err(());
                }
                None => {
                    usage(&program);
                    eprintln!("{}: <query> was not provided", "ERROR".red());
                    return Err(());
                }
            }
        }
        "add" => {
            match args.next().as_deref() {
                Some("-j") => {
                    let journal_name = args.next();

                    if journal_name.is_none() {
                        usage(&program);
                        eprintln!("{}: New journal name was not provided", "ERROR".red());
                        return Err(());
                    }

                    let journal_name = journal_name.unwrap();

                    let mut journals: Vec<Journal> = Vec::new();
                    get_journals(filepath, &mut journals)?;

                    let new_log: Log = make_log(&journal_name)?;
                    let mut new_journal: Journal = Journal::new(&journal_name);
                    new_journal.add_log(new_log);
                    journals.push(new_journal);

                    let json_content = serde_json::to_string(&journals).map_err(|err| {
                        eprintln!("{}: Could not parse journal struct into json file: {err}", "ERROR".red())
                    })?;

                    update_json(json_content, filepath)?;
                    println!("{}", format!("Sucessfully created journal").green());
                    return Ok(());
                }
                Some(user_journal_query) => {
                    let mut journals: Vec<Journal> = Vec::new();
                    get_journals(filepath, &mut journals)?;
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
                            for journal in journals.iter_mut() {
                                if journal.name == user_journal_query {
                                    journal.add_log(new_log.clone());
                                }
                            }
                            let json_content = serde_json::to_string(&journals).map_err(|err| {
                                eprintln!("{}: Could not parse journal struct into json file: {err}", "ERROR".red())
                            })?;

                            update_json(json_content, filepath)?;
                            println!("{}", format!("Sucessfully added log into {user_journal_query}").green());
                            return Ok(());
                        }
                    }
                },
                None => {
                    eprintln!("{}: Journal name to query was not provided", "ERROR".red());
                    return Err(());
                }
            }
        }
        "remove" => {
            match args.next().as_deref() {
                Some("-j") => {
                    let input_journal_name = args.next();

                    if input_journal_name.is_none() {
                        usage(&program);
                        eprintln!("{}: Journal name was not provided", "ERROR".red());
                        return Err(());
                    }
                    let input_journal_name = input_journal_name.unwrap();
                    let mut journals: Vec<Journal> = Vec::new();
                    get_journals(filepath, &mut journals)?;

                    let mut found = false;
                    for (i, journal) in journals.iter().enumerate() {
                        if journal.name == input_journal_name {
                            journals.remove(i);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        eprintln!("{}", format!("Journal with <{input_journal_name}> name not found").red());
                        return Err(());
                    }

                    let json_content = serde_json::to_string(&journals).map_err(|err| {
                        eprintln!("{}: Could not parse journal struct into json file: {err}", "ERROR".red())
                    })?;

                    update_json(json_content, filepath)?;
                    println!("{}", format!("Sucessfully removed {input_journal_name} journal").green());
                    return Ok(());
                },

                Some(input_uid) => {
                    let mut journals: Vec<Journal> = Vec::new();
                    get_journals(filepath, &mut journals)?;
                    let mut found = false;

                    for journal in journals.iter_mut() {
                        let logs = &mut journal.logs;
                        for (i, log) in logs.iter_mut().enumerate() {
                            if log.uid == input_uid {
                                logs.remove(i);
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        eprintln!("{}", format!("Log with <{input_uid}> name not found").red());
                        return Err(());
                    }

                    let json_content = serde_json::to_string(&journals).map_err(|err| {
                        eprintln!("{}: Could not parse journal struct into json file: {err}", "ERROR".red())
                    })?;

                    update_json(json_content, filepath)?;
                    println!("{}", format!("Sucessfully removed log with {input_uid} UID").green());
                    return Ok(());
                },
                None => {
                    eprintln!("{}: log name was not provided", "ERROR".red());
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
