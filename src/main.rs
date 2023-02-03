use colored::Colorize;
use std::env;
use std::process::ExitCode;
use std::result::Result;
use crate::stu::{utils::*, Journal, Log};

mod stu;

static DEFAULT_EDITOR: &str = "vim";
static mut SORT: bool = false;

fn setup() -> Result<(), ()> {
    let filepath: &str = &setup_data()?;

    let mut args = env::args();
    args.next().unwrap();

    let subcommand = args.next().ok_or_else(|| {
        usage();
        eprintln!("{}: Subcommand is needed", "ERROR".red());
    })?;

    match subcommand.as_str() {
        "-h" | "--help" => {
            usage();
            return Ok(());
        }
        "show" => {
            let mut journals: Vec<Journal> = Vec::new();
            stu::get_journals(filepath, &mut journals)?;

            if journals.len() == 0 {
                eprintln!(
                    "{}",
                    format!(
                        "There's no journals at the moment, create one with\
                            the command `stu -j add <name>`"
                    )
                    .red()
                );

                return Err(());
            }
            match args.next().as_deref() {
                Some("-m") => {
                    stu::show_metrics(&journals);
                    return Ok(());
                }
                None => {
                    stu::show_journals(&mut journals);
                    return Ok(());
                }
                Some(_) => {
                    eprintln!("{}: Unknown argument", "ERROR".red());
                    return Err(());
                }
            }
        }
        "get" => {
            let value = args.next();
            match value {
                Some(mut str) => {
                    if is_string_numeric(&str) {
                        return stu::query_uid(&str, filepath);
                    }

                    if str == "-s" {
                        match args.next() {
                            Some(new_str) => unsafe {
                                SORT = true;
                                str = new_str;
                            },
                            None => {
                                eprintln!("ERROR: Unknown argument");
                                return Err(());
                            }
                        }
                    }

                    if is_string_alphanumeric(&str) {
                        return stu::query_for(&str.to_lowercase(), filepath);
                    }

                    eprintln!("{}: Unknown query type", "ERROR".red());
                    return Err(());
                }
                None => {
                    eprintln!("{}", format!("<query> was not provided").red());
                    return Err(());
                }
            }
        }
        "add" => match args.next().as_deref() {
            Some("-j") => {
                let journal_name = args.next();

                if journal_name.is_none() {
                    eprintln!("{}", format!("New journal name was not provided").red());
                    return Err(());
                }

                let journal_name = journal_name.unwrap();

                let mut journals: Vec<Journal> = Vec::new();
                stu::get_journals(filepath, &mut journals)?;

                let new_log: Log = stu::make_log(&journal_name)?;
                let mut new_journal: Journal = Journal::new(&journal_name);
                new_journal.add_log(new_log);
                journals.push(new_journal);

                let json_content = serde_json::to_string(&journals).map_err(|err| {
                    eprintln!(
                        "{}: Could not parse journal struct into json file: {err}",
                        "ERROR".red()
                    )
                })?;

                stu::sync_data(json_content, filepath)?;
                println!("{}", format!("Sucessfully created journal").green());
                return Ok(());
            }
            Some(user_journal_query) => {
                let mut journals: Vec<Journal> = Vec::new();
                stu::get_journals(filepath, &mut journals)?;
                let result = journals
                    .iter()
                    .filter(|x| x.name == user_journal_query)
                    .next();
                match result {
                    None => {
                        stu::list_journals(&journals);
                        let text = format!(
                            r"{text1}{name}{text2} {prompt}",
                            text1 = "Journal with the name `".red(),
                            name = user_journal_query.red(),
                            text2 = "` was not found, do you \
                                           want to create one? "
                                .red(),
                            prompt = "[y/n]"
                        );
                        println!("{text}");
                    }

                    Some(_) => {
                        let new_log = stu::make_log(user_journal_query)?;
                        for journal in journals.iter_mut() {
                            if journal.name == user_journal_query {
                                journal.add_log(new_log.clone());
                            }
                        }
                        let json_content = serde_json::to_string(&journals).map_err(|err| {
                            eprintln!(
                                "{}: Could not parse journal struct into json file: {err}",
                                "ERROR".red()
                            )
                        })?;

                        stu::sync_data(json_content, filepath)?;
                        println!(
                            "{}",
                            format!("Sucessfully added log into {user_journal_query}").green()
                        );
                        return Ok(());
                    }
                }
            }
            None => {
                eprintln!(
                    "{}", format!(
                        "Journal name was not provided, run `stu show` to list available journals")
                        .red());
                return Err(());
            }
        },
        "remove" => match args.next().as_deref() {
            Some("-j") => {
                let input_journal_name = args.next();

                if input_journal_name.is_none() {
                    eprintln!("{}",format!("Journal name was not provided").red());
                    return Err(());
                }
                let input_journal_name = input_journal_name.unwrap();
                let mut journals: Vec<Journal> = Vec::new();
                stu::get_journals(filepath, &mut journals)?;

                let mut found = false;
                for (i, journal) in journals.iter().enumerate() {
                    if journal.name == input_journal_name {
                        journals.remove(i);
                        found = true;
                        break;
                    }
                }
                if !found {
                    eprintln!(
                        "{}",
                        format!("Journal with <{input_journal_name}> name not found").red()
                    );
                    return Err(());
                }

                let json_content = serde_json::to_string(&journals).map_err(|err| {
                    eprintln!(
                        "{}: Could not parse journal struct into json file: {err}",
                        "ERROR".red()
                    )
                })?;

                stu::sync_data(json_content, filepath)?;
                println!(
                    "{}",
                    format!("Sucessfully removed {input_journal_name} journal").green()
                );
                return Ok(());
            }

            Some(input_uid) => {
                let mut journals: Vec<Journal> = Vec::new();
                stu::get_journals(filepath, &mut journals)?;
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
                    eprintln!(
                        "{}: Could not parse journal struct into json file: {err}",
                        "ERROR".red()
                    )
                })?;

                stu::sync_data(json_content, filepath)?;
                println!(
                    "{}",
                    format!("Sucessfully removed log with {input_uid} UID").green()
                );
                return Ok(());
            }
            None => {
                eprintln!("{}: log name was not provided", "ERROR".red());
                return Err(());
            }
        },
        _ => {
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

// TODO: add edit option
