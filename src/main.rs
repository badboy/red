/// red - A Rust Editor
///
/// An `ed` clone, written in Rust.

extern crate rustyline;
extern crate exitfailure;
#[macro_use] extern crate failure;

use exitfailure::ExitFailure;
use rustyline::error::ReadlineError;
use rustyline::Editor;

#[derive(Debug,Eq,PartialEq)]
enum Action {
    Quit,
    Continue,
    Unknown,
}

fn dispatch(cmd: &str) -> Result<Action, failure::Error> {
    match cmd.trim() {
        "" => Ok(Action::Unknown),
        "q" => Ok(Action::Quit),
        _ => Ok(Action::Unknown),
    }
}

fn main() -> Result<(), ExitFailure> {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline("");
        match readline {
            Ok(line) => {
                match dispatch(&line)? {
                    Action::Quit => break,
                    Action::Continue => {},
                    Action::Unknown => {
                        println!("?");
                    },
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("?");
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                Err(format_err!("Error: {:?}", err))?;
            }
        }
    }

    Ok(())
}
