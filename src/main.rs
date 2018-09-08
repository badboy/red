/// red - A Rust Editor
///
/// An `ed` clone, written in Rust.

extern crate rustyline;
extern crate exitfailure;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
extern crate env_logger;

use exitfailure::ExitFailure;
use rustyline::error::ReadlineError;
use rustyline::Editor;

#[derive(Debug,Eq,PartialEq)]
enum Action {
    Quit,
    Continue,
    Unknown,
}

#[derive(Debug)]
enum Range {
    From(usize),
    Full(usize, usize),
    Single(usize),
    Jump(usize),
}


impl Range {
    fn iter(&self) -> Box<Iterator<Item=usize>> {
        match self {
            Range::From(n) => Box::new(*n..),
            Range::Full(a, b) => Box::new(*a..=*b),
            Range::Single(n) => Box::new(*n..=*n),
            Range::Jump(_) => panic!("Can't iterate a jump"),
        }
    }
}

#[derive(Debug)]
struct Red {
    current_line: usize,
    total_lines: usize,
    data: Vec<String>,
}

impl Red {
    fn new() -> Red {
        let data = vec![
            "line 1".into(),
            "line 2".into(),
            "line 3".into(),
            "line 4".into(),
            "line 5".into(),
            "line 6".into(),
            "line 7".into(),
            "line 8".into(),
        ];

        Red {
            current_line: data.len(),
            total_lines: data.len(),
            data: data
        }
    }

    fn set_line(&mut self, line: usize) -> Result<(), failure::Error> {
        if line < 1 || line > self.total_lines {
            Err(format_err!("Out of bounds jump"))
        } else {
            self.current_line = line;
            Ok(())
        }
    }

    fn dispatch(&mut self, cmd: &str) -> Result<Action, failure::Error> {
        let cmd = cmd.trim();
        let (cmd, range) = parse_range(self.current_line, self.total_lines, cmd);
        debug!("cmd: {:?}, range: {:?}", cmd, range);

        match cmd {
            "" => {
                if let Range::Jump(idx) = range {
                    self.set_line(idx)?;
                    debug!("After jump, printing line {}", self.current_line);
                    println!("{}", self.data[self.current_line-1]);
                    return Ok(Action::Continue);
                }

                if self.current_line == self.total_lines {
                    Ok(Action::Unknown)
                } else {
                    let line = self.current_line+1;
                    self.set_line(line)?;
                    debug!("Printing line {}", self.current_line);
                    println!("{}", self.data[self.current_line-1]);
                    Ok(Action::Continue)
                }
            }
            "q" => Ok(Action::Quit),
            "p" => {
                debug!("Printing lines in range {:?}", range);
                for idx in range.iter().take(self.total_lines) {
                    println!("{}", self.data[idx-1]);
                }
                Ok(Action::Continue)
            }
            "n" => {
                debug!("Printing numbered lines in range {:?}", range);
                for idx in range.iter().take(self.total_lines) {
                    println!("{}\t{}", idx, self.data[idx-1]);
                }
                Ok(Action::Continue)
            }
            "w" => {
                debug!("Writing lines in range {:?}", range);
                Ok(Action::Continue)
            }
            _ => Ok(Action::Unknown),
        }
    }
}

fn valid_range_char(c: char) -> bool {
    match c {
        '%' | '$' | '.' => true,
        _ if c.is_numeric() => true,
        _ => false,
    }
}

fn build_range(left: &str, right: &str, current_line: usize, total_lines: usize) -> Range {
    if left.is_empty() && right.is_empty() {
        return Range::From(1);
    }

    if left.is_empty() {
        if right == "$" || right == "%" {
            return Range::From(1);
        }

        if right == "." {
            return Range::Full(1, current_line);
        }

        let right = right.parse::<usize>().expect("Need valid right end");
        return Range::Full(1, right);
    }

    if right.is_empty() {
        if left == "." {
            return Range::Single(current_line);

        }

        // Special case for $p -> last line only
        if left == "$" || right == "%" {
            return Range::Single(total_lines);
        }

        let left = left.parse::<usize>().expect("Need valid left end");
        return Range::Single(left);
    }

    let left = match left {
        "." => current_line,
        _ => left.parse::<usize>().expect("Need valid left end"),
    };

    let right = match right {
        "." => current_line,
        "$" | "%" => return Range::From(left),
        _ => right.parse::<usize>().expect("Need valid right end"),
    };

    Range::Full(left, right)
}

fn parse_range(current_line: usize, total_lines: usize, line: &str) -> (&str, Range) {
    if line.is_empty() {
        return (line, Range::Single(current_line));
    }

    let comma_idx = line.find(',');

    if let Some(comma_idx) = comma_idx {
        let left = &line[0..comma_idx];
        let rest = &line[comma_idx+1..];
        let rest_end = rest.find(|c| !valid_range_char(c));
        let (right,rest) = match rest_end {
            Some(idx) => (&rest[0..idx], &rest[idx..]),
            None => (rest, ""),
        };
        debug!("left: {:?}", left);
        debug!("right: {:?}", right);
        debug!("rest: {:?}", rest);

        return (rest, build_range(left, right, current_line, total_lines));
    }

    let cmd_idx = line.find(|c| !valid_range_char(c));
    debug!("cmd idx: {:?}", cmd_idx);

    if let Some(cmd_idx) = cmd_idx {
        if cmd_idx == 0 {
            return (line, Range::Single(current_line))
        } else {
            let range = build_range(&line[0..cmd_idx], "", current_line, total_lines);
            return (&line[cmd_idx..], range);
        }
    }

    let first_char = line.chars().next().unwrap();
    if valid_range_char(first_char) {
        if first_char == '%' || first_char == '$' {
            return ("", Range::Jump(total_lines));
        } else {
            let line = line.parse::<usize>().expect("Need an integer to jump to");
            return ("", Range::Jump(line));
        }
    }

    (line, Range::Single(current_line))
}

fn main() -> Result<(), ExitFailure> {
    env_logger::init();

    let mut rl = Editor::<()>::new();
    let mut ed = Red::new();

    loop {
        debug!("Ed: {:?}", ed);
        let readline = rl.readline("*");
        match readline {
            Ok(line) => {
                debug!("Line: {:?}", line);
                let cmd = ed.dispatch(&line)?;
                debug!("Command: {:?}", cmd);
                match cmd {
                    Action::Quit => break,
                    Action::Continue => {},
                    Action::Unknown => {
                        println!("?");
                    },
                }
            },
            Err(ReadlineError::Interrupted) => {
                debug!("Readline Interrupted");
                println!("?");
            },
            Err(ReadlineError::Eof) => {
                debug!("EOF send. Quitting.");
                break
            },
            Err(err) => {
                debug!("Unknown error. Failing.");
                Err(format_err!("Error: {:?}", err))?;
            }
        }
    }

    Ok(())
}
