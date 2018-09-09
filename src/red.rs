use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

use failure;
use commands::{Mode, Command, Action};
use tokenizer;
use parser;

#[derive(Debug)]
pub struct Red {
    prompt: String,
    pub current_line: usize,
    pub total_lines: usize,
    pub data: Vec<String>,
    pub mode: Mode,
    pub path: Option<String>,
    pub dirty: bool,
    pub last_error: Option<String>
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

        Red {
            prompt: prompt,
            current_line: data.len(),
            total_lines: data.len(),
            data: data,
            mode: Mode::Command,
            path: path,
            dirty: false,
            last_error: None,
        }
    }

    pub fn data_size(&self) -> usize {
        self.data.iter().map(|l| l.len()+1).sum()
    }

    pub fn set_line(&mut self, line: usize) -> Result<(), failure::Error> {
        if line < 1 || line > self.total_lines {
            Err(format_err!("Invalid address"))
        } else {
            self.current_line = line;
            Ok(())
        }
    }

    pub fn get_line(&self, line: usize) -> Option<&str> {
        if line > 0 && line <= self.total_lines {
            Some(&self.data[line-1])
        } else {
            None
        }
    }

    fn parse_command(&self, line: &str) -> Result<Command, failure::Error> {
        let tokens = tokenizer::tokenize(line)?;
        debug!("tokens: {:#?}", tokens);
        let command = parser::parse(&tokens)?;
        debug!("parsed command: {:#?}", command);

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
        if self.data.is_empty() {
            self.data.push(line.into());
        } else {
            self.data.insert(idx, line.into());
        }
        self.current_line += 1;
        self.total_lines = self.data.len();
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

