use colored::Colorize;
use std::env;
use std::process::Command;
use std::result::Result;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Write;

pub fn get_date() -> String {
    let date_process = Command::new("/usr/bin/date")
        .arg("+%m/%d/%Y")
        .output()
        .expect("ERROR: Could not run date process");

    let output = std::str::from_utf8(&date_process.stdout)
        .unwrap_or_else(|_| "unknown")
        .trim();
    return output.to_string();
}

pub fn get_percentage(amount: f32, total: f32) -> f32 {
    let result = (amount * 100.0) / total;
    let rounded = result.round();
    return rounded;
}

pub fn edit_text(filepath: String) -> Result<(), ()> {
    // TODO: Give user the external option of choosing the EDITOR
    let env_editor: String =
        env::var("EDITOR").unwrap_or_else(|_| crate::DEFAULT_EDITOR.to_string());

    let mut editor_process = Command::new(&env_editor)
        .arg(filepath)
        .spawn()
        .expect(format!("ERROR: Could not start {} editor", env_editor).as_str());

    let _exit_code = editor_process.wait().map_err(|err| {
        eprintln!("{}: {err}", "ERROR".red());
    })?;

    Ok(())
}
pub fn remove_brackets(string: &str) -> String {
    string
        .chars()
        .filter(|c| c != &'[' && c != &']')
        .collect::<String>()
}

pub fn is_string_numeric(str: &str) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }

    return true;
}

pub fn is_string_alphanumeric(str: &str) -> bool {
    for c in str.chars() {
        if c != '/' && c != '-' && !c.is_alphanumeric() && !c.is_whitespace() {
            return false;
        }
    }
    return true;
}

pub fn usage() {
    println!(
        "{usage}: stu <subcommand> <options>\n",
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

pub fn setup_data() -> Result<String, ()> {
    let home_path = env::var("HOME").map_err(|_| {
        eprintln!("{}: You got no home lol", "ERROR".red());
    })?;

    let data_dir_path = &format!("{home_path}/.local/share/stu/");

    if !Path::new(data_dir_path).exists() {
        fs::create_dir(data_dir_path).map_err(|err| {
            eprintln!("{}: Could not create database file: {err}", "ERROR".red());
        })?;
    }

    let data_file_path = format!("{data_dir_path}data.json");

    if !Path::new(&data_file_path).exists() {
        let mut file = File::create(&data_file_path).map_err(|err| {
            eprintln!("{}: Could not create database file: {err}", "ERROR".red());
        })?;

        writeln!(file, "[\n]").unwrap();
    }

    Ok(data_file_path)
}
