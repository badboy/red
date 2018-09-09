use failure;
use std::cmp;
use std::fs::{self, File};
use std::io::{self, Write};
use Red;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Address {
    CurrentLine,
    LastLine,
    Numbered(usize),
    Offset(isize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Command,
    Input,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Quit,
    Continue,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Noop,
    Quit,
    Help,
    Jump {
        address: Address,
    },
    Print {
        start: Option<Address>,
        end: Option<Address>,
    },
    Numbered {
        start: Option<Address>,
        end: Option<Address>,
    },
    Delete {
        start: Option<Address>,
        end: Option<Address>,
    },
    Write {
        start: Option<Address>,
        end: Option<Address>,
        file: Option<String>,
    },
    Insert {
        before: Option<Address>,
    },
    Append {
        after: Option<Address>,
    },
    Edit {
        file: Option<String>,
    },
    Change {
        start: Option<Address>,
        end: Option<Address>,
    },
    Read {
        after: Option<Address>,
        file: Option<String>,
    },
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
            Delete { start, end } => Self::delete(ed, start, end),
            Write { start, end, file } => Self::write(ed, start, end, file),
            Insert { before } => Self::insert(ed, before),
            Append { after } => Self::append(ed, after),
            Edit { file } => Self::edit(ed, file),
            Change { start, end } => Self::change(ed, start, end),
            Read { after, file } => Self::read(ed, after, file),
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

    fn help(ed: &mut Red) -> Result<Action, failure::Error> {
        if let Some(error) = ed.last_error.as_ref() {
            println!("{}", error);
        }
        Ok(Action::Continue)
    }

    fn quit(ed: &mut Red) -> Result<Action, failure::Error> {
        if ed.dirty {
            ed.dirty = false;
            Err(format_err!("Warning: buffer modified"))
        } else {
            Ok(Action::Quit)
        }
    }

    fn jump(ed: &mut Red, addr: Address) -> Result<Action, failure::Error> {
        use self::Address::*;
        match addr {
            CurrentLine => { /* Don't jump at all */ }
            LastLine => {
                let new_line = ed.total_lines;
                ed.set_line(new_line)?
            }
            Numbered(n) => ed.set_line(n)?,
            Offset(n) => {
                let new_line = ed.current_line as isize + n;
                if new_line < 1 {
                    return Err(format_err!("Invalid address"));
                }
                ed.set_line(new_line as usize)?;
            }
        }

        // After a jump, print the current line
        Self::print(ed, None, None)
    }

    fn print(
        ed: &mut Red,
        start: Option<Address>,
        end: Option<Address>,
    ) -> Result<Action, failure::Error> {
        let stdout = io::stdout();
        let handle = stdout.lock();
        Self::write_range(handle, ed, start, end, false)
    }

    fn numbered(
        ed: &mut Red,
        start: Option<Address>,
        end: Option<Address>,
    ) -> Result<Action, failure::Error> {
        let stdout = io::stdout();
        let handle = stdout.lock();
        Self::write_range(handle, ed, start, end, true)
    }

    fn delete(
        ed: &mut Red,
        start: Option<Address>,
        end: Option<Address>,
    ) -> Result<Action, failure::Error> {
        if ed.data.is_empty() {
            return Err(format_err!("Invalid address"));
        }

        match (start, end) {
            (None, None) => {
                let line = ed.current_line;
                ed.data.remove(line - 1);
                ed.dirty = true;
                ed.total_lines = ed.data.len();
                ed.current_line = cmp::min(line, ed.data.len());
            }

            (Some(start), None) => {
                let line = Self::get_actual_line(&ed, start)?;
                ed.data.remove(line - 1);
                ed.dirty = true;
                ed.total_lines = ed.data.len();
                ed.current_line = cmp::min(line, ed.data.len());
            }

            (None, Some(end)) => {
                let end = Self::get_actual_line(&ed, end)?;

                for _ in 1..=end {
                    ed.data.remove(0);
                }

                ed.dirty = true;
                ed.total_lines = ed.data.len();
                ed.current_line = cmp::min(end, ed.data.len());
            }

            (Some(start), Some(end)) => {
                let start = Self::get_actual_line(&ed, start)?;
                let end = Self::get_actual_line(&ed, end)?;

                for _ in start..=end {
                    ed.data.remove(start - 1);
                }

                ed.dirty = true;
                ed.total_lines = ed.data.len();
                ed.current_line = cmp::min(start, ed.data.len());
            }
        }
        Ok(Action::Continue)
    }

    fn write(
        ed: &mut Red,
        mut start: Option<Address>,
        mut end: Option<Address>,
        file: Option<String>,
    ) -> Result<Action, failure::Error> {
        let file = file.or_else(|| ed.path.clone());
        match file {
            None => Ok(Action::Unknown),
            Some(path) => {
                // By default, write the whole buffer
                if start.is_none() && end.is_none() {
                    start = Some(Address::Numbered(1));
                    end = Some(Address::LastLine);
                }

                debug!("Writing to file {:?} ({:?}..{:?})", path, start, end);

                let file = File::create(&path)?;
                Self::write_range(file, ed, start, end, false)?;
                let size = fs::metadata(&path)?.len();
                println!("{}", size);

                ed.path = Some(path);
                ed.dirty = false;

                Ok(Action::Continue)
            }
        }
    }

    fn insert(ed: &mut Red, before: Option<Address>) -> Result<Action, failure::Error> {
        let mut addr = before
            .map(|addr| Self::get_actual_line(&ed, addr))
            .unwrap_or_else(|| Ok(ed.current_line))?;
        // Insert after the previous line
        if addr > 1 {
            addr -= 1;
        }
        ed.current_line = addr;
        ed.mode = Mode::Input;
        Ok(Action::Continue)
    }

    fn append(ed: &mut Red, after: Option<Address>) -> Result<Action, failure::Error> {
        let addr = after
            .map(|addr| Self::get_actual_line(&ed, addr))
            .unwrap_or_else(|| Ok(ed.current_line))?;
        ed.current_line = addr;
        ed.mode = Mode::Input;
        Ok(Action::Continue)
    }

    fn edit(ed: &mut Red, file: Option<String>) -> Result<Action, failure::Error> {
        let file = file.or_else(|| ed.path.clone());

        let file = match file {
            None => return Err(format_err!("No current filename")),
            Some(file) => file,
        };
        ed.load_file(file)?;

        Ok(Action::Continue)
    }

    fn change(
        ed: &mut Red,
        start: Option<Address>,
        end: Option<Address>,
    ) -> Result<Action, failure::Error> {
        Self::delete(ed, start, end)?;
        let mut addr = ed.current_line;
        if addr > 1 {
            addr -= 1;
        }
        ed.set_line(addr)?;
        ed.mode = Mode::Input;
        ed.dirty = true;
        Ok(Action::Continue)
    }

    fn read(
        ed: &mut Red,
        after: Option<Address>,
        file: Option<String>,
    ) -> Result<Action, failure::Error> {
        let file = file.or_else(|| ed.path.clone());

        let file = match file {
            None => return Err(format_err!("No current filename")),
            Some(file) => file,
        };
        let data = ed.load_data(&file)?;

        let mut addr = after
            .map(|addr| Self::get_actual_line(&ed, addr))
            .unwrap_or_else(|| Ok(ed.current_line))?;

        let mut written = 0;
        for line in data {
            written += line.len() + 1;
            if ed.data.is_empty() {
                ed.data.push(line);
            } else {
                ed.data.insert(addr, line);
            }
            addr += 1;
        }

        ed.dirty = true;
        ed.total_lines = ed.data.len();
        ed.current_line = addr;
        println!("{}", written);

        Ok(Action::Continue)
    }

    fn write_range<W: Write>(
        mut output: W,
        ed: &mut Red,
        start: Option<Address>,
        end: Option<Address>,
        show_number: bool,
    ) -> Result<Action, failure::Error> {
        if ed.data.is_empty() {
            return Err(format_err!("Invalid address"));
        }

        match (start, end) {
            (None, None) => {
                if show_number {
                    write!(output, "{}\t", ed.current_line)?;
                }
                writeln!(output, "{}", ed.get_line(ed.current_line).unwrap())?;
            }

            (Some(start), None) => {
                ed.current_line = Self::get_actual_line(&ed, start)?;

                if show_number {
                    write!(output, "{}\t", ed.current_line)?;
                }
                writeln!(output, "{}", ed.get_line(ed.current_line).unwrap())?;
            }

            (None, Some(end)) => {
                let end = Self::get_actual_line(&ed, end)?;

                for line in 1..=end {
                    if show_number {
                        write!(output, "{}\t", line)?;
                    }
                    writeln!(output, "{}", ed.get_line(line).unwrap())?;
                }

                ed.current_line = end;
            }

            (Some(start), Some(end)) => {
                let start = Self::get_actual_line(&ed, start)?;
                let end = Self::get_actual_line(&ed, end)?;

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

    fn get_actual_line(ed: &Red, addr: Address) -> Result<usize, failure::Error> {
        use self::Address::*;
        match addr {
            CurrentLine => Ok(ed.current_line),
            LastLine => Ok(ed.total_lines),
            Numbered(n) => {
                if n > ed.total_lines {
                    return Err(format_err!("Invalid address"));
                }
                Ok(n)
            }
            Offset(n) => {
                let line = ed.current_line as isize + n;
                if line < 1 {
                    return Err(format_err!("Invalid address"));
                }

                let line = line as usize;
                if line > ed.total_lines {
                    return Err(format_err!("Invalid address"));
                }

                Ok(line)
            }
        }
    }
}
