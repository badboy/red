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

pub fn tokenize(line: &str) -> Vec<Token> {
    let mut res = vec![];

    let command_idx = line.find(|c: char| {
        COMMANDS.contains(&c)
    });
    debug!("2: command idx: {:?}", command_idx);

    let addr_part = match command_idx {
        None => line,
        Some(idx) => &line[0..idx]
    };
    debug!("2: addr part: {:?}", addr_part);

    let addr_separator_idx = addr_part.find(|c| {
        [',', ';'].contains(&c)
    });
    debug!("2: addr sep idx: {:?}", addr_separator_idx);

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
    debug!("2: rest addr: {:?}", rest_addr);
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
            panic!("Need a suffix character or whitespace");
        }

        let arg = line[after_cmd_idx+1..].trim();
        if !arg.is_empty() {
            res.push(Token::Argument(arg));
        }
    }

    res
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let expected : Vec<Token>= vec![];
        assert_eq!(expected, tokenize(""));
    }

    #[test]
    fn single_address() {
        let expected = vec![
            Token::Address("1"),
        ];

        assert_eq!(expected, tokenize("1"));
    }

    #[test]
    fn lower_address() {
        let expected = vec![
            Token::Address("1"),
            Token::Separator(','),
        ];

        assert_eq!(expected, tokenize("1,"));
    }

    #[test]
    fn upper_address() {
        let expected = vec![
            Token::Separator(','),
            Token::Address("$"),
        ];

        assert_eq!(expected, tokenize(",$"));
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

        assert_eq!(expected, tokenize("1,$pn"));
    }

    #[test]
    fn only_command() {
        let expected = vec![
            Token::Command('p'),
        ];

        assert_eq!(expected, tokenize("p"));
    }

    #[test]
    fn command_with_suffix() {
        let expected = vec![
            Token::Command('p'),
            Token::Suffix('n'),
        ];

        assert_eq!(expected, tokenize("pn"));
    }

    #[test]
    fn command_with_arg() {
        let expected = vec![
            Token::Command('p'),
            Token::Argument("file.txt"),
        ];

        assert_eq!(expected, tokenize("p file.txt"));
    }

    #[test]
    #[should_panic]
    // FIXME: tokenize should return a Result
    fn missing_whitespace_argument() {
        tokenize("pfile.txt");
    }
}