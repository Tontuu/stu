#[cfg(test)]
mod tests {
    use crate::stu::utils;

    #[test]
    fn test_get_date() {
        utils::get_date();
        let result = utils::get_date();
        let expected: String;
        if cfg!(windows) {
            let date = std::process::Command::new("cmd").args(["/C", "date /t"]).output().expect("Could not use date on tests");
            expected = std::str::from_utf8(&date.stdout).unwrap_or_else(|_| "unknown").trim().to_string();
        } else {
            let date = std::process::Command::new("/usr/bin/date").arg("+%m/%d/%Y").output().expect("ERROR: Could not run date process");
            expected = std::str::from_utf8(&date.stdout).unwrap_or_else(|_| "unknown").trim().to_string();
        }
        assert_eq!(result, expected);
    }

    #[test]
    fn test_edit_text() {
        let mut tf = tempfile::Builder::new()
            .prefix("stu-log_")
            .suffix(".txt")
            .rand_bytes(4)
            .tempfile()
            .unwrap();

        let note_builder_text = "type 'UWU' inside brackets: []";

        use std::io::{Read, Seek, Write};
        write!(tf, "{}", &note_builder_text).unwrap();
        tf.flush().unwrap();

        utils::edit_text(tf.path().display().to_string()).unwrap();

        tf.flush().unwrap();
        tf.rewind().unwrap();

        let mut result = String::new();
        tf.read_to_string(&mut result).unwrap();

        println!("{}", result);

        let expected = "type 'UWU' inside brackets: [UWU]";
        assert_eq!(result, expected);

    }


}

pub mod stu {
    pub mod utils;
}