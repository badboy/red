use std::fs::File;
use std::io::{self, Write};
use failure;
use {Red};

#[derive(Debug,PartialEq,Eq)]
pub enum Address {
    CurrentLine,
    LastLine,
    Numbered(usize),
    Offset(isize),
}

#[derive(Debug)]
pub enum Mode {
    Command,
    Input,
}

#[derive(Debug,Eq,PartialEq)]
pub enum Action {
    Quit,
    Continue,
    Unknown,
}

#[derive(Debug,PartialEq,Eq)]
pub enum Command {
    Noop,
    Quit,
    Help,
    Jump { address: Address },
    Print { start: Option<Address>, end: Option<Address> },
    Numbered { start: Option<Address>, end: Option<Address> },
    Delete { start: Option<Address>, end: Option<Address> },
    Write { start: Option<Address>, end: Option<Address>, file: Option<String> },
    Insert { before: Option<Address> },
    Append { after: Option<Address> },
}

impl Command {
    pub fn execute(self, ed: &mut Red) -> Result<Action, failure::Error> {
        debug!("Command::execute: {:?}", self);
        use Command::*;

        match self {
            Noop => Self::noop(ed),
            Help => Self::help(ed),
            Quit => Self::quit(ed),
            Jump { address } => Self::jump(ed, address),
            Print { start, end } => Self::print(ed, start, end),
            Numbered { start, end } => Self::numbered(ed, start, end),
            Write { start, end, file } => Self::write(ed, start, end, file),
            Insert { before } => Self::insert(ed, before),
            Append { after } => Self::append(ed, after),
            _ => Ok(Action::Unknown),
        }
    }

    fn noop(ed: &mut Red) -> Result<Action, failure::Error> {
        if ed.current_line < ed.total_lines {
            ed.current_line += 1;
            Self::print(ed, None, None)
        } else {
            Ok(Action::Unknown)
        }
    }

    fn help(_ed: &mut Red) -> Result<Action, failure::Error> {
        Ok(Action::Continue)
    }

    fn quit(_ed: &mut Red) -> Result<Action, failure::Error> {
        Ok(Action::Quit)
    }

    fn jump(ed: &mut Red, addr: Address) -> Result<Action, failure::Error> {
        use self::Address::*;
        match addr {
            CurrentLine => { /* Don't jump at all */ },
            LastLine    => ed.current_line = ed.total_lines,
            Numbered(n) => ed.current_line = n,
            Offset(n)   => {
                let current = ed.current_line as isize + n;
                ed.current_line = current as usize;
            },
        }

        // After a jump, print the current line
        Self::print(ed, None, None)
    }

    fn print(ed: &mut Red, start: Option<Address>, end: Option<Address>) -> Result<Action, failure::Error> {
        let stdout = io::stdout();
        let handle = stdout.lock();
        Self::write_range(handle, ed, start, end, false)
    }

    fn numbered(ed: &mut Red, start: Option<Address>, end: Option<Address>) -> Result<Action, failure::Error> {
        let stdout = io::stdout();
        let handle = stdout.lock();
        Self::write_range(handle, ed, start, end, true)
    }


    fn write(ed: &mut Red, mut start: Option<Address>, mut end: Option<Address>, file: Option<String>)
        -> Result<Action, failure::Error> {

        let file = file.or(ed.path.take());
        match file {
            None => return Ok(Action::Unknown),
            Some(path) => {

                // By default, write the whole buffer
                if start.is_none() && end.is_none() {
                    start = Some(Address::Numbered(1));
                    end   = Some(Address::LastLine);
                }

                debug!("Writing to file {:?} ({:?}..{:?})", path, start, end);

                let file = File::create(&path)?;
                Self::write_range(file, ed, start, end, false)?;

                ed.path = Some(path);
                ed.dirty = false;

                Ok(Action::Continue)
            }
        }
    }

    fn insert(ed: &mut Red, before: Option<Address>) -> Result<Action, failure::Error> {
        let mut addr = before.map(|addr| Self::get_actual_line(&ed, addr)).unwrap_or(ed.current_line);
        // Insert after the previous line
        if addr > 1 {
            addr -= 1;
        }
        ed.current_line = addr;
        ed.mode = Mode::Input;
        Ok(Action::Continue)
    }

    fn append(ed: &mut Red, after: Option<Address>) -> Result<Action, failure::Error> {
        let addr = after.map(|addr| Self::get_actual_line(&ed, addr)).unwrap_or(ed.current_line);
        ed.current_line = addr;
        ed.mode = Mode::Command;
        Ok(Action::Continue)
    }

    fn write_range<W: Write>(mut output: W,
                             ed: &mut Red,
                             start: Option<Address>, end: Option<Address>, show_number: bool)
        -> Result<Action, failure::Error> {
        match (start, end) {
            (None, None) => {
                if show_number {
                    write!(output, "{}\t", ed.current_line)?;
                }
                writeln!(output, "{}", ed.get_line(ed.current_line).unwrap())?;
            },

            (Some(start), None) => {
                ed.current_line = Self::get_actual_line(&ed, start);

                if show_number {
                    write!(output, "{}\t", ed.current_line)?;
                }
                writeln!(output, "{}", ed.get_line(ed.current_line).unwrap())?;
            }

            (None, Some(end)) => {
                let end = Self::get_actual_line(&ed, end);

                for line in 1..=end {
                    if show_number {
                        write!(output, "{}\t", line)?;
                    }
                    writeln!(output, "{}", ed.get_line(line).unwrap())?;
                }

                ed.current_line = end;

            }

            (Some(start), Some(end)) => {
                let start = Self::get_actual_line(&ed, start);
                let end = Self::get_actual_line(&ed, end);

                for line in start..=end {
                    if show_number {
                        write!(output, "{}\t", line)?;
                    }
                    writeln!(output, "{}", ed.get_line(line).unwrap())?;
                }

                ed.current_line = end;

            }
        }

        Ok(Action::Continue)
    }

    fn get_actual_line(ed: &Red, addr: Address) -> usize {
        use self::Address::*;
        match addr {
            CurrentLine => ed.current_line,
            LastLine => ed.total_lines,
            Numbered(n) => n,
            Offset(n) => {
                let current = ed.current_line as isize + n;
                current as usize
            },
        }

    }
}
