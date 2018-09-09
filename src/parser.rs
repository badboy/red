use failure;

use commands::Address;
use commands::Command;
use tokenizer::Token;

fn parse_address(addr: &str) -> Result<Address, failure::Error> {
    match addr {
        "." => return Ok(Address::CurrentLine),
        "$" => return Ok(Address::LastLine),
        _ => {}
    }

    if &addr[0..1] == "+" || &addr[0..1] == "-" {
        let n = addr[0..]
            .parse::<isize>()
            .map_err(|_| format_err!("Invalid address"))?;
        return Ok(Address::Offset(n));
    }

    let n = addr
        .parse::<usize>()
        .map_err(|_| format_err!("Invalid address"))?;
    Ok(Address::Numbered(n))
}

pub fn parse(tokens: &[Token]) -> Result<Command, failure::Error> {
    if tokens.is_empty() {
        return Ok(Command::Noop);
    }

    let mut start = None;
    let mut end = None;
    let mut arg = None;
    let mut cmd = None;
    let mut first_addr = false;
    let mut separator_found = false;

    for token in tokens {
        match token {
            Token::Address(addr) if !first_addr => {
                start = Some(parse_address(addr)?);
                first_addr = true;
            }
            Token::Address(addr) if first_addr => {
                end = Some(parse_address(addr)?);
            }
            Token::Separator(_) => {
                separator_found = true;
                first_addr = true;
            }
            Token::Argument(a) => {
                arg = Some(a.to_string());
            }
            Token::Command(c) => {
                cmd = Some(c);
            }
            _ => {}
        }
    }

    // If there was a separator, fix up the range to cover all
    if separator_found && start.is_none() && end.is_none() {
        start = Some(Address::Numbered(1));
        end = Some(Address::LastLine);
    }

    let cmd = match cmd {
        None if start.is_some() && end.is_none() => {
            return Ok(Command::Jump {
                address: start.unwrap(),
            });
        }
        None => return Ok(Command::Noop),
        Some(c) => c,
    };

    let cmd = match cmd {
        'p' => Command::Print { start, end },
        'n' => Command::Numbered { start, end },
        'd' => Command::Delete { start, end },
        'w' => Command::Write {
            start,
            end,
            file: arg,
        },
        'i' => Command::Insert {
            before: start.or(end),
        },
        'a' => Command::Append {
            after: end.or(start),
        },
        'h' => Command::Help,
        'q' => Command::Quit,
        'e' => Command::Edit { file: arg },
        'c' => Command::Change { start, end },
        _ => Command::Noop,
    };
    Ok(cmd)
}

#[cfg(test)]
mod test {
    use super::*;
    use tokenizer::tokenize;

    #[test]
    fn address_variants() {
        assert_eq!(Address::CurrentLine, parse_address(".").unwrap());
        assert_eq!(Address::LastLine, parse_address("$").unwrap());
        assert_eq!(Address::Numbered(23), parse_address("23").unwrap());
        assert_eq!(Address::Offset(2), parse_address("+2").unwrap());
        assert_eq!(Address::Offset(-3), parse_address("-3").unwrap());
    }

    #[test]
    #[should_panic]
    fn wrong_address_format() {
        parse_address("d").unwrap();
    }

    #[test]
    fn parse_empty() {
        assert_eq!(Command::Noop, parse(&tokenize("").unwrap()).unwrap());
    }

    #[test]
    fn parse_addr_print() {
        assert_eq!(
            Command::Print {
                start: Some(Address::Numbered(1)),
                end: Some(Address::Numbered(2))
            },
            parse(&tokenize("1,2p").unwrap()).unwrap()
        );
    }

    #[test]
    fn parse_write() {
        assert_eq!(
            Command::Write {
                start: None,
                end: None,
                file: Some("file.txt".into())
            },
            parse(&tokenize("w file.txt").unwrap()).unwrap()
        );
    }

    #[test]
    fn parse_append() {
        assert_eq!(
            Command::Append {
                after: Some(Address::Numbered(2)),
            },
            parse(&tokenize("1,2a").unwrap()).unwrap()
        );

        assert_eq!(
            Command::Append { after: None },
            parse(&tokenize("a").unwrap()).unwrap()
        );

        assert_eq!(
            Command::Append {
                after: Some(Address::Numbered(1)),
            },
            parse(&tokenize("1a").unwrap()).unwrap()
        );
    }

    #[test]
    fn parse_jumps() {
        assert_eq!(
            Command::Jump {
                address: Address::Numbered(2)
            },
            parse(&tokenize("2").unwrap()).unwrap()
        );

        assert_eq!(
            Command::Jump {
                address: Address::CurrentLine
            },
            parse(&tokenize(".").unwrap()).unwrap()
        );
    }
}
