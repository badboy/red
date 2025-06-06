use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use failure::format_err;

use crate::commands::{Action, Command, Mode};
use crate::parser;
use crate::tokenizer;

#[derive(Debug)]
pub struct Red {
    prompt: String,
    pub current_line: usize,
    pub data: Vec<String>,
    pub mode: Mode,
    pub path: Option<String>,
    pub dirty: bool,
    pub last_error: Option<String>,
}

impl Red {
    pub fn new(prompt: String, path: Option<String>) -> Red {
        let (path, data) = match path {
            None => (None, vec![]),
            Some(path) => {
                let data = match File::open(&path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        reader.lines().map(|l| l.unwrap()).collect()
                    }
                    Err(_) => vec![],
                };
                (Some(path), data)
            }
        };

        let len = data.len();
        Red {
            prompt,
            data,
            path,
            current_line: len,
            mode: Mode::Command,
            dirty: false,
            last_error: None,
        }
    }

    pub fn load_data(&self, path: &str) -> Result<Vec<String>, failure::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data = reader.lines().map(|l| l.unwrap()).collect();
        Ok(data)
    }

    pub fn load_file(&mut self, path: String) -> Result<(), failure::Error> {
        let data = self.load_data(&path)?;
        let len = data.len();
        self.path = Some(path);
        self.data = data;
        self.current_line = len;

        Ok(())
    }

    pub fn data_size(&self) -> usize {
        self.data.iter().map(|l| l.len() + 1).sum()
    }

    pub fn lines(&self) -> usize {
        self.data.len()
    }

    pub fn set_line(&mut self, line: usize) -> Result<(), failure::Error> {
        if line < 1 || line > self.lines() {
            Err(format_err!("Invalid address"))
        } else {
            self.current_line = line;
            Ok(())
        }
    }

    pub fn get_line(&self, line: usize) -> Option<&str> {
        if line > 0 && line <= self.lines() {
            Some(&self.data[line - 1])
        } else {
            None
        }
    }

    fn parse_command(&self, line: &str) -> Result<Command, failure::Error> {
        let tokens = tokenizer::tokenize(line)?;
        log::debug!("tokens: {:#?}", tokens);
        let command = parser::parse(&tokens)?;
        log::debug!("parsed command: {:#?}", command);

        Ok(command)
    }

    fn dispatch_command(&mut self, line: &str) -> Result<Action, failure::Error> {
        let command = self.parse_command(line.trim())?;
        command.execute(self)
    }

    fn dispatch_input(&mut self, line: &str) -> Result<Action, failure::Error> {
        if line == "." {
            self.mode = Mode::Command;
            return Ok(Action::Continue);
        }

        let idx = self.current_line;
        log::debug!("Inserting line at {}", idx);
        if self.data.is_empty() {
            self.data.push(line.into());
        } else {
            self.data.insert(idx, line.into());
        }
        self.current_line += 1;
        self.dirty = true;

        Ok(Action::Continue)
    }

    pub fn dispatch(&mut self, line: &str) -> Result<Action, failure::Error> {
        match self.mode {
            Mode::Command => self.dispatch_command(line),
            Mode::Input => self.dispatch_input(line),
        }
    }

    pub fn prompt(&self) -> &str {
        match self.mode {
            Mode::Command => &self.prompt,
            Mode::Input => "",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_edits() {
        let mut ed = Red::new("".into(), None);

        assert_eq!(Mode::Command, ed.mode);
        ed.dispatch("i").unwrap();
        assert_eq!(Mode::Input, ed.mode);
        ed.dispatch("Some light text.").unwrap();
        ed.dispatch(".").unwrap();
        assert_eq!(Mode::Command, ed.mode);

        let data = &ed.data;
        assert_eq!("Some light text.", data[0]);
    }

    #[test]
    fn complex_stuff() {
        let mut ed = Red::new("".into(), None);

        ed.dispatch("i").unwrap();
        ed.dispatch("Line 1.").unwrap();
        ed.dispatch("Line 2.").unwrap();
        ed.dispatch("Line 4.").unwrap();
        ed.dispatch(".").unwrap();
        ed.dispatch("3i").unwrap();
        ed.dispatch("Line 3.").unwrap();
        ed.dispatch(".").unwrap();
        ed.dispatch("0a").unwrap();
        ed.dispatch("Line 0.").unwrap();
        ed.dispatch(".").unwrap();

        {
            let data = &ed.data;
            assert_eq!("Line 0.", data[0]);
            assert_eq!("Line 1.", data[1]);
            assert_eq!("Line 2.", data[2]);
            assert_eq!("Line 3.", data[3]);
            assert_eq!("Line 4.", data[4]);
        }

        ed.dispatch("3d").unwrap();

        {
            let data = &ed.data;
            assert_eq!("Line 0.", data[0]);
            assert_eq!("Line 1.", data[1]);
            assert_eq!("Line 3.", data[2]);
            assert_eq!("Line 4.", data[3]);
        }
    }

    #[test]
    fn change_line() {
        let mut ed = Red::new("".into(), None);

        ed.dispatch("a").unwrap();
        ed.dispatch("Line 1").unwrap();
        ed.dispatch("Line 2").unwrap();
        ed.dispatch("Line 3").unwrap();
        ed.dispatch("Line 4").unwrap();
        ed.dispatch(".").unwrap();

        {
            ed.dispatch("2c").unwrap();
            ed.dispatch("world").unwrap();
            ed.dispatch(".").unwrap();

            let data = &ed.data;
            assert_eq!(vec!["Line 1", "world", "Line 3", "Line 4"], &data[..]);
        }

        {
            ed.dispatch("1c").unwrap();
            ed.dispatch("hello").unwrap();
            ed.dispatch(".").unwrap();

            let data = &ed.data;
            assert_eq!(vec!["hello", "world", "Line 3", "Line 4"], &data[..]);
        }
    }
}
