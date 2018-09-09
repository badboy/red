/// red - A Rust Editor
///
/// An `ed` clone, written in Rust.

extern crate rustyline;
extern crate exitfailure;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
extern crate env_logger;
#[macro_use]
extern crate structopt;

use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

use exitfailure::ExitFailure;
use structopt::StructOpt;
use rustyline::error::ReadlineError;
use rustyline::Editor;

mod commands;
mod tokenizer;
mod parser;

use commands::{Mode, Command, Action};

/// Command line parser.
#[derive(Debug, StructOpt)]
pub struct Cli {
  /// file
  path: Option<String>,
  /// use STRING as an interactive prompt
  #[structopt(short = "p", long = "prompt", default_value="")]
  prompt: String,
}

#[derive(Debug)]
pub struct Red {
    prompt: String,
    current_line: usize,
    total_lines: usize,
    data: Vec<String>,
    mode: Mode,
    path: Option<String>,
    dirty: bool,
    last_error: Option<String>
}

impl Red {
    fn new(prompt: String, path: Option<String>) -> Red {
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

    fn data_size(&self) -> usize {
        self.data.iter().map(|l| l.len()+1).sum()
    }

    fn set_line(&mut self, line: usize) -> Result<(), failure::Error> {
        if line < 1 || line > self.total_lines {
            Err(format_err!("Invalid address"))
        } else {
            self.current_line = line;
            Ok(())
        }
    }

    fn get_line(&self, line: usize) -> Option<&str> {
        if line > 0 && line <= self.total_lines {
            Some(&self.data[line-1])
        } else {
            None
        }
    }

    fn parse_command(&self, line: &str) -> Result<Command, failure::Error> {
        let tokens = tokenizer::tokenize(line)?;
        debug!("tokens: {:#?}", tokens);
        let command = parser::parse(&tokens);
        debug!("command: {:#?}", command);

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

    fn dispatch(&mut self, line: &str) -> Result<Action, failure::Error> {
        match self.mode {
            Mode::Command => self.dispatch_command(line),
            Mode::Input => self.dispatch_input(line),
        }
    }

    fn prompt(&self) -> &str {
        match self.mode {
            Mode::Command => &self.prompt,
            Mode::Input => "",
        }
    }
}

fn main() -> Result<(), ExitFailure> {
    env_logger::init();

    let args = Cli::from_args();
    let mut rl = Editor::<()>::new();
    let mut ed = Red::new(args.prompt, args.path);

    let size = ed.data_size();
    if size > 0 {
        println!("{}", size);
    }

    loop {
        debug!("Ed: {:?}", ed);
        let readline = rl.readline(ed.prompt());
        match readline {
            Ok(line) => {
                debug!("Line: {:?}", line);
                match ed.dispatch(&line) {
                    Ok(res) => {
                        debug!("Result: {:?}", res);

                        match res {
                            Action::Quit => break,
                            Action::Continue => {},
                            Action::Unknown => {
                                println!("?");
                            }
                        }
                    }
                    Err(err) => {
                        debug!("Saving error: {:?}", err);
                        ed.last_error = Some(err.to_string());
                        println!("?");
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                debug!("Readline Interrupted");
                println!("?");
            },
            Err(ReadlineError::Eof) => {
                debug!("EOF send.");
                match Command::Quit.execute(&mut ed) {
                    Err(err) => {
                        ed.last_error = Some(err.to_string());
                        println!("?");
                    }
                    Ok(Action::Quit) => break,
                    Ok(_) => panic!("Unknown action on EOF"),
                }
            },
            Err(err) => {
                debug!("Unknown error. Failing.");
                Err(format_err!("Error: {:?}", err))?;
            }
        }
    }

    Ok(())
}
