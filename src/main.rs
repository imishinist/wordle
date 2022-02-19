use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::Parser;

#[derive(Debug)]
struct CharPosition {
    char: char,
    position: usize,
}

#[derive(Debug)]
struct Filter {
    length: usize,
    ignore_chars: Vec<char>,
    char_positions: Vec<CharPosition>,
    different_char_positions: Vec<CharPosition>,
}

impl Filter {
    fn new(
        ignore_chars: Vec<char>,
        char_positions: Vec<CharPosition>,
        different_char_positions: Vec<CharPosition>,
    ) -> Self {
        Self {
            length: 5,
            ignore_chars,
            char_positions,
            different_char_positions,
        }
    }

    fn accept(&self, word: &str) -> bool {
        if word.len() != self.length {
            return false;
        }

        for c in self.ignore_chars.iter() {
            if word.find(*c).is_some() {
                return false;
            }
        }

        if !self.accept_char_position(word) {
            return false;
        }

        if !self.accept_char(word) {
            return false;
        }

        true
    }

    // accept_char_position returns whether or not char_positions matches all the rules.
    fn accept_char_position(&self, word: &str) -> bool {
        let pos_char = word.char_indices().collect::<HashMap<_, _>>();
        for cp in self.char_positions.iter() {
            match pos_char.get(&cp.position) {
                Some(c) if *c == cp.char => {}
                _ => return false,
            }
        }
        for cp in self.different_char_positions.iter() {
            match pos_char.get(&cp.position) {
                Some(c) if *c == cp.char => return false,
                _ => {}
            }
        }

        true
    }

    fn accept_char(&self, word: &str) -> bool {
        for cp in self.different_char_positions.iter() {
            if word.find(cp.char).is_none() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::{CharPosition, Filter};

    #[test]
    fn filter_test() {
        let ignores = vec!['a', 'b', 'c'];
        let filter = Filter::new(ignores, vec![], vec![]);

        assert!(!filter.accept("word"));
        assert!(!filter.accept("audio"));
        assert!(filter.accept("write"));

        let ignores = vec!['a', 'b', 'c'];
        let char_positions = vec![
            CharPosition {
                char: 'd',
                position: 0,
            },
            CharPosition {
                char: 'e',
                position: 4,
            },
        ];
        let filter = Filter::new(ignores, char_positions, vec![]);
        assert!(!filter.accept("avoid"));
        assert!(!filter.accept("wheel"));
        assert!(!filter.accept("false"));
        assert!(!filter.accept("dirty"));
        assert!(filter.accept("drive"));

        let ignores = vec!['a', 'b', 'c'];
        let char_positions = vec![
            CharPosition {
                char: 'd',
                position: 0,
            },
            CharPosition {
                char: 'e',
                position: 4,
            },
        ];
        let different_char_positions = vec![CharPosition {
            char: 'r',
            position: 1,
        }];
        let filter = Filter::new(ignores, char_positions, different_char_positions);
        assert!(!filter.accept("avoid"));
        assert!(!filter.accept("wheel"));
        assert!(!filter.accept("false"));
        assert!(!filter.accept("dirty"));
        assert!(!filter.accept("drive"));
        assert!(!filter.accept("dense"));
        assert!(filter.accept("doree"));
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    target: Option<String>,

    #[clap(short, long)]
    ignore_chars: Option<String>,

    #[clap(short, long)]
    different_positions: Option<Vec<String>>,
}

fn parse_char_position(target: String) -> Vec<CharPosition> {
    let mut ret = Vec::new();

    if target.len() != 5 {
        return ret;
    }
    for (pos, c) in target.as_str().char_indices() {
        if c == '*' {
            continue;
        }
        ret.push(CharPosition {
            char: c,
            position: pos,
        });
    }

    ret
}

fn parse_ignore_chars(ignore_chars: String) -> Vec<char> {
    ignore_chars.chars().collect()
}

fn parse_different_positions(targets: Vec<String>) -> Vec<CharPosition> {
    let mut ret = Vec::new();

    for target in targets.iter() {
        if target.len() != 5 {
            continue;
        }

        for (pos, c) in target.as_str().char_indices() {
            if c == '*' {
                continue;
            }
            ret.push(CharPosition {
                char: c,
                position: pos,
            })
        }
    }

    ret
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let char_position = match args.target {
        Some(t) => parse_char_position(t),
        None => Vec::new(),
    };
    let ignore_chars = match args.ignore_chars {
        Some(t) => parse_ignore_chars(t),
        None => Vec::new(),
    };
    let not_match_char_position = match args.different_positions {
        Some(t) => parse_different_positions(t),
        None => Vec::new(),
    };
    let filter = Filter::new(ignore_chars, char_position, not_match_char_position);

    let dict_path = env::var("DICT_PATH").unwrap_or_else(|_| "/usr/share/dict/words".into());
    let file = File::open(dict_path)?;
    let lines = BufReader::new(file).lines();
    for line in lines {
        match line {
            Ok(line) if filter.accept(line.to_lowercase().as_str()) => {
                println!("{}", line);
            }
            _ => continue,
        }
    }

    Ok(())
}
