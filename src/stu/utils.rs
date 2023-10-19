use colored::Colorize;
use std::process::Command;
use std::result::Result;
use std::path::Path;
use std::fs::File;
use std::io::Write;

pub fn get_date() -> String {
    let date_process: std::process::Output;
    if cfg!(windows) {
        date_process = Command::new("cmd")
        .args(["/C", "date /t"])
        .output()
        .expect("ERROR: Could no trun date process on windows");
    } else {
        date_process = Command::new("/usr/bin/date")
            .arg("+%m/%d/%Y")
            .output()
            .expect("ERROR: Could not run date process");

    }
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
    edit::edit_file(filepath).map_err(|err| {
        eprintln!("{}: Could not edit file: {err}", "ERROR");
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
    println!("{usage}: stu <subcommand> <options>\n", usage = "Usage".red());
    println!("Change editor with `EDITOR=emacs` for instance. Default editor is vim\n");
    println!("{subcommands}:", subcommands = "Subcommands".red());
    println!("    -h      --help                    print help");
    println!();
    println!("    show   <subcommand>               print all user journals, use -m if you wanna print the metrics");
    println!("                ╰------------------------> print metrics: \"-m\"");
    println!();
    println!("    add    <subcommand> <value>       add either a new log or journal");
    println!("                ╰------------------------> add journal: \"-j\"");
    println!();
    println!("    remove <subcommand> <value>       remove a log with the given <value>");
    println!("                │          ╰-------------> value can be: [UID, journal]");
    println!("                ╰------------------------> remove journal: \"-j\"");
    println!();
    println!("    get    <subcommand> <query>       search for <query> and print results");
    println!("                │          ╰-------------> query can be: [UID, journal, subject, topic, \"MM/DD/YYYY\"]");
    println!("                ╰------------------------> sort query: \"-s\"");
    println!();
    println!("    edit   <UID>                      edit log with the given UID");
    println!();
}

pub fn setup_data() -> Result<String, ()> {
    let home_path:String = simple_home_dir::home_dir().unwrap().display().to_string();
    let data_dir_path = if cfg!(windows) { home_path + "\\stu\\" } else { "/local/share/stu/".to_string() };

    if !std::path::Path::new(&data_dir_path).exists() {
        std::fs::create_dir(&data_dir_path).map_err(|err| {
            eprintln!("{}: Could not create database file: {err}", "ERROR");
        }).unwrap();
    }

    let data_file_path = format!("{data_dir_path}data.json", );

    if !Path::new(&data_file_path).exists() {
        let mut file = File::create(&data_file_path).map_err(|err| {
            eprintln!("{}: Could not create database file: {err}", "ERROR".red());
        })?;

        writeln!(file, "[\n]").unwrap();
    }

    Ok(data_file_path)
}
