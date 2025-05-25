//! red - A Rust Editor
//!
//! An `ed` clone, written in Rust.

use exitfailure::ExitFailure;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use structopt::StructOpt;
use failure::format_err;

mod commands;
mod parser;
mod red;
mod tokenizer;

use commands::{Action, Command};
use red::Red;

/// A Rust Editor.
#[derive(Debug, StructOpt)]
pub struct Cli {
    /// file
    path: Option<String>,
    /// use STRING as an interactive prompt
    #[structopt(short = "p", long = "prompt", default_value = "")]
    prompt: String,
}

fn main() -> Result<(), ExitFailure> {
    env_logger::init();

    let args = Cli::from_args();
    let mut rl = DefaultEditor::new()?;
    let mut ed = Red::new(args.prompt, args.path);

    let size = ed.data_size();
    if size > 0 {
        println!("{}", size);
    }

    loop {
        log::debug!("Ed: {:?}", ed);
        let readline = rl.readline(ed.prompt());
        match readline {
            Ok(line) => {
                log::debug!("Line: {:?}", line);
                match ed.dispatch(&line) {
                    Ok(res) => {
                        log::debug!("Result: {:?}", res);

                        match res {
                            Action::Quit => break,
                            Action::Continue => {}
                            Action::Unknown => {
                                println!("?");
                            }
                        }
                    }
                    Err(err) => {
                        log::debug!("Saving error: {:?}", err);
                        ed.last_error = Some(err.to_string());
                        println!("?");
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                log::debug!("Readline Interrupted");
                println!("?");
            }
            Err(ReadlineError::Eof) => {
                log::debug!("EOF send.");
                let cmd = Command::Quit { force: false };
                match cmd.execute(&mut ed) {
                    Err(err) => {
                        ed.last_error = Some(err.to_string());
                        println!("?");
                    }
                    Ok(Action::Quit) => break,
                    Ok(_) => panic!("Unknown action on EOF"),
                }
            }
            Err(err) => {
                log::debug!("Unknown error: {:?}", err);
                Err(format_err!("Error: {:?}", err))?;
            }
        }
    }

    Ok(())
}
