use failure;

static COMMANDS: &'static [char] = &[
    'p', // print
    'n', // numbered print
    'w', // write [arg]
    'd', // delete
    'a', // append
    'i', // insert
    'c', // replace line
    'h', // show last error
    'q', // quit
    'Q', // Force-quit
    'e', // edit file
    'c', // change
    'r', // read
    'm', // move
    's', // substitute
    'g', // global
];

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Address(&'a str),
    Separator(char),
    Command(char),
    Suffix(&'a str),
    Argument(&'a str),
}

pub fn tokenize(line: &str) -> Result<Vec<Token>, failure::Error> {
    let mut res = vec![];

    let command_idx = line.find(|c: char| COMMANDS.contains(&c));
    debug!("command idx: {:?}", command_idx);

    let addr_part = match command_idx {
        None => line,
        Some(idx) => &line[0..idx],
    };
    debug!("addr part: {:?}", addr_part);

    let addr_separator_idx = addr_part.find(|c| [',', ';'].contains(&c));
    debug!("addr sep idx: {:?}", addr_separator_idx);

    let rest_addr = match addr_separator_idx {
        None => addr_part,
        Some(idx) => {
            let addr = &addr_part[..idx];
            if !addr.is_empty() {
                res.push(Token::Address(addr));
            }
            let sep = addr_part[idx..].chars().next().unwrap();
            res.push(Token::Separator(sep));
            &addr_part[idx + 1..]
        }
    };
    debug!("rest addr: {:?}", rest_addr);
    if !rest_addr.is_empty() {
        res.push(Token::Address(rest_addr));
    }

    let after_cmd_idx = match command_idx {
        None => line.len(),
        Some(idx) => {
            let cmd = &line[idx..=idx];
            let cmd = cmd.chars().next().unwrap();
            res.push(Token::Command(cmd));
            idx + 1
        }
    };

    if after_cmd_idx < line.len() {
        let suffix_char = line[after_cmd_idx..=after_cmd_idx].chars().next().unwrap();
        if suffix_char == ' ' {
            let arg = line[after_cmd_idx + 1..].trim();
            if !arg.is_empty() {
                res.push(Token::Argument(arg));
            }
        } else {
            let arg = &line[after_cmd_idx..];
            let before_arg = arg.find(|c| c == ' ');
            match before_arg {
                None => res.push(Token::Suffix(arg)),
                Some(idx) => {
                    let suffix = &arg[..idx];
                    res.push(Token::Suffix(suffix));

                    let arg = &arg[idx + 1..];
                    if arg.len() > 0 {
                        res.push(Token::Argument(arg));
                    }
                }
            }
        }
    }

    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let expected: Vec<Token> = vec![];
        assert_eq!(expected, tokenize("").unwrap());
    }

    #[test]
    fn single_address() {
        let expected = vec![Token::Address("1")];

        assert_eq!(expected, tokenize("1").unwrap());
    }

    #[test]
    fn lower_address() {
        let expected = vec![Token::Address("1"), Token::Separator(',')];

        assert_eq!(expected, tokenize("1,").unwrap());
    }

    #[test]
    fn upper_address() {
        let expected = vec![Token::Separator(','), Token::Address("$")];

        assert_eq!(expected, tokenize(",$").unwrap());
    }

    #[test]
    fn tokenize_with_full_address() {
        let expected = vec![
            Token::Address("1"),
            Token::Separator(','),
            Token::Address("$"),
            Token::Command('p'),
            Token::Suffix("n"),
        ];

        assert_eq!(expected, tokenize("1,$pn").unwrap());
    }

    #[test]
    fn only_command() {
        let expected = vec![Token::Command('p')];

        assert_eq!(expected, tokenize("p").unwrap());
    }

    #[test]
    fn command_with_suffix() {
        let expected = vec![Token::Command('p'), Token::Suffix("n")];

        assert_eq!(expected, tokenize("pn").unwrap());
    }

    #[test]
    fn command_with_arg() {
        let expected = vec![Token::Command('p'), Token::Argument("file.txt")];

        assert_eq!(expected, tokenize("p file.txt").unwrap());
    }

    #[test]
    fn move_cmd() {
        let expected = vec![
            Token::Address("1"),
            Token::Separator(','),
            Token::Address("2"),
            Token::Command('m'),
            Token::Suffix("3"),
        ];
        assert_eq!(expected, tokenize("1,2m3").unwrap());
    }

    #[test]
    fn address_command_suffix_arg() {
        let expected = vec![
            Token::Address("1"),
            Token::Separator(','),
            Token::Address("2"),
            Token::Command('m'),
            Token::Suffix("3"),
            Token::Argument("param"),
        ];
        assert_eq!(expected, tokenize("1,2m3 param").unwrap());
    }
}
