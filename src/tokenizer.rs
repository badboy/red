use failure;

static COMMANDS : &'static [char] = &[
    'p', // print
    'n', // numbered print
    'w', // write [arg]
    'd', // delete
    'a', // append
    'i', // insert
    'c', // replace line
    'h', // show last error
    'q', // quit
];

static SUFFIXES : &'static [char] = &[
    'l',
    'p',
    'n',
];

#[derive(Debug,PartialEq,Eq)]
pub enum Token<'a> {
    Address(&'a str),
    Separator(char),
    Command(char),
    Suffix(char),
    Argument(&'a str),
}

pub fn tokenize(line: &str) -> Result<Vec<Token>, failure::Error> {
    let mut res = vec![];

    let command_idx = line.find(|c: char| {
        COMMANDS.contains(&c)
    });
    debug!("command idx: {:?}", command_idx);

    let addr_part = match command_idx {
        None => line,
        Some(idx) => &line[0..idx]
    };
    debug!("addr part: {:?}", addr_part);

    let addr_separator_idx = addr_part.find(|c| {
        [',', ';'].contains(&c)
    });
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
            &addr_part[idx+1..]
        }
    };
    debug!("rest addr: {:?}", rest_addr);
    if !rest_addr.is_empty() {
        res.push(Token::Address(rest_addr));
    }

    let after_cmd_idx = match command_idx {
        None => line.len(),
        Some(idx) => {
            let cmd = &line[idx..idx+1];
            let cmd = cmd.chars().next().unwrap();
            res.push(Token::Command(cmd));
            idx+1
        }
    };

    if after_cmd_idx < line.len() {
        let suffix_char = line[after_cmd_idx..after_cmd_idx+1].chars().next().unwrap();
        if SUFFIXES.contains(&suffix_char) {
            res.push(Token::Suffix(suffix_char));
        } else if suffix_char != ' ' {
            return Err(format_err!("Invalid command suffix"));
        }

        let arg = line[after_cmd_idx+1..].trim();
        if !arg.is_empty() {
            res.push(Token::Argument(arg));
        }
    }

    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let expected : Vec<Token>= vec![];
        assert_eq!(expected, tokenize("").unwrap());
    }

    #[test]
    fn single_address() {
        let expected = vec![
            Token::Address("1"),
        ];

        assert_eq!(expected, tokenize("1").unwrap());
    }

    #[test]
    fn lower_address() {
        let expected = vec![
            Token::Address("1"),
            Token::Separator(','),
        ];

        assert_eq!(expected, tokenize("1,").unwrap());
    }

    #[test]
    fn upper_address() {
        let expected = vec![
            Token::Separator(','),
            Token::Address("$"),
        ];

        assert_eq!(expected, tokenize(",$").unwrap());
    }

    #[test]
    fn tokenize_with_full_address() {
        let expected = vec![
            Token::Address("1"),
            Token::Separator(','),
            Token::Address("$"),
            Token::Command('p'),
            Token::Suffix('n'),
        ];

        assert_eq!(expected, tokenize("1,$pn").unwrap());
    }

    #[test]
    fn only_command() {
        let expected = vec![
            Token::Command('p'),
        ];

        assert_eq!(expected, tokenize("p").unwrap());
    }

    #[test]
    fn command_with_suffix() {
        let expected = vec![
            Token::Command('p'),
            Token::Suffix('n'),
        ];

        assert_eq!(expected, tokenize("pn").unwrap());
    }

    #[test]
    fn command_with_arg() {
        let expected = vec![
            Token::Command('p'),
            Token::Argument("file.txt"),
        ];

        assert_eq!(expected, tokenize("p file.txt").unwrap());
    }

    #[test]
    #[should_panic]
    fn missing_whitespace_argument() {
        tokenize("pfile.txt").unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_suffix() {
        tokenize("hello world").unwrap();
    }
}
