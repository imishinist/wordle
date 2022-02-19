use bit_set::BitSet;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use clap::{Parser, Subcommand};
use regex::Regex;

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
    use crate::{CharFreq, CharPosition, Filter, WordScore};
    use std::collections::BinaryHeap;

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

    #[test]
    fn char_freq_test() {
        let mut char_freq = CharFreq::new();

        char_freq.add_char('a');
        char_freq.add_char('a');
        char_freq.add_char('b');
        char_freq.add_char('c');
        char_freq.add_char('\n');
        char_freq.add_char('-');
        assert_eq!(
            char_freq
                .to_vec()
                .into_iter()
                .filter(|(_, count)| *count > 0)
                .collect::<Vec<(char, usize)>>(),
            vec![('a', 2), ('b', 1), ('c', 1),]
        );
    }

    #[test]
    fn word_score_test() {
        let mut char_freq = CharFreq::new();
        char_freq.add_char('a');
        char_freq.add_char('a');
        char_freq.add_char('a');
        char_freq.add_char('b');
        char_freq.add_char('c');
        let word_score = WordScore::new("aaaaaaabc".to_string(), &char_freq);
        assert_eq!(word_score.score, 3 + 1 + 1);
    }

    #[test]
    fn word_score_order() {
        let mut heap = BinaryHeap::new();

        let mut char_freq = CharFreq::new();
        char_freq.add_char('a');
        char_freq.add_char('a');
        char_freq.add_char('a');
        char_freq.add_char('b');
        char_freq.add_char('c');

        heap.push(WordScore::new("abc".to_string(), &char_freq));
        heap.push(WordScore::new("cba".to_string(), &char_freq));
        heap.push(WordScore::new("bcd".to_string(), &char_freq));

        let ws = heap.pop().unwrap();
        assert_eq!(ws.score, 5);
        assert_eq!(ws.word, "abc".to_string());
        let ws = heap.pop().unwrap();
        assert_eq!(ws.score, 5);
        assert_eq!(ws.word, "cba".to_string());
        let ws = heap.pop().unwrap();
        assert_eq!(ws.score, 2);
        assert_eq!(ws.word, "bcd".to_string());

        assert_eq!(heap.pop(), None);
    }
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

#[derive(Debug, Eq, PartialEq)]
struct CharFreq {
    inner: Vec<usize>,
}

impl CharFreq {
    fn new() -> Self {
        let inner = (0..26).map(|_| 0usize).collect();
        Self { inner }
    }

    #[inline]
    fn add_char(&mut self, c: char) {
        if !c.is_ascii_alphabetic() {
            return;
        }
        let c = c.to_ascii_lowercase() as usize - 'a' as usize;

        let entry = unsafe { self.inner.get_unchecked_mut(c) };
        *entry += 1;
    }

    #[inline]
    fn get_freq(&self, c: char) -> usize {
        let c = c.to_ascii_lowercase() as usize - 'a' as usize;
        unsafe { *self.inner.get_unchecked(c) }
    }

    #[inline]
    fn to_vec(&self) -> Vec<(char, usize)> {
        let mut v = self.inner.iter().enumerate().collect::<Vec<_>>();
        v.sort_by(|(c1, &count1), (c2, &count2)| {
            count1
                .cmp(&count2)
                .then_with(|| c1.cmp(c2).reverse())
                .reverse()
        });

        v.iter()
            .map(|(c, count)| ((*c as u8 + b'a') as char, **count))
            .collect()
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let file = File::open(path).unwrap();
        let lines = BufReader::new(file).lines();

        let re = Regex::new(r"^([a-z]):(\d+)$").unwrap();

        let mut char_freq = Self::new();
        for line in lines.flatten() {
            for cap in re.captures_iter(line.as_str()) {
                let char = cap.get(1).map(|c| c.as_str()).expect("invalid format");
                let count = cap
                    .get(2)
                    .map(|c| c.as_str().parse::<usize>())
                    .expect("invalid format")
                    .expect("invalid format");
                let c = char.as_bytes()[0] as usize - 'a' as usize;
                let entry = unsafe { char_freq.inner.get_unchecked_mut(c) };
                *entry = count;
            }
        }
        char_freq
    }
}

#[derive(Debug, Eq, PartialEq)]
struct WordScore<'a> {
    word: String,
    score: usize,

    freqs: &'a CharFreq,
}

impl<'a> WordScore<'a> {
    fn new(word: String, char_freq: &'a CharFreq) -> Self {
        let mut ws = Self {
            word,
            score: 0,

            freqs: char_freq,
        };

        ws.calc_score();
        ws
    }

    fn calc_score(&mut self) {
        let mut s = BitSet::with_capacity(26);
        for c in self.word.chars() {
            let c = c.to_ascii_lowercase();
            let c = c as u8 - b'a';
            s.insert(c as usize);
        }

        for x in s.iter() {
            let freq = self.freqs.get_freq((x as u8 + b'a') as char);
            self.score += freq;
        }
    }
}

impl<'a> PartialOrd for WordScore<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for WordScore<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then_with(|| self.word.cmp(&other.word).reverse())
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Grep {
        target: Option<String>,

        #[clap(short, long)]
        ignore_chars: Option<String>,

        #[clap(short, long)]
        different_positions: Option<Vec<String>>,

        #[clap(short, long)]
        score_sort: Option<usize>,
    },
    Analyse {},
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let dict_path = env::var("DICT_PATH").unwrap_or_else(|_| "/usr/share/dict/words".into());
    let char_freq_path = env::var("CHAR_FREQ_PATH").unwrap_or_else(|_| {
        let current = env::current_dir().unwrap_or_else(|_| "/tmp".into());
        current
            .join("char.freq")
            .into_os_string()
            .into_string()
            .unwrap()
    });

    match &cli.command {
        Commands::Grep {
            target,
            ignore_chars,
            different_positions,
            score_sort,
        } => {
            let char_position = match target {
                Some(t) => parse_char_position(t.to_string()),
                None => Vec::new(),
            };
            let ignore_chars = match ignore_chars {
                Some(t) => parse_ignore_chars(t.to_string()),
                None => Vec::new(),
            };
            let not_match_char_position = match different_positions {
                Some(t) => parse_different_positions(t.clone()),
                None => Vec::new(),
            };
            let filter = Filter::new(ignore_chars, char_position, not_match_char_position);

            let file = File::open(dict_path)?;
            let lines = BufReader::new(file).lines();

            let out = stdout();
            let mut out = BufWriter::new(out.lock());

            match score_sort {
                Some(k) => {
                    let char_freq = CharFreq::from_file(char_freq_path);
                    let mut k = *k as isize;

                    let mut heap = BinaryHeap::with_capacity(k as usize);
                    for line in lines {
                        match line {
                            Ok(line) if filter.accept(line.to_lowercase().as_str()) => {
                                heap.push(WordScore::new(line, &char_freq));
                            }
                            _ => continue,
                        }
                    }

                    while let Some(ws) = heap.pop() {
                        if k <= 0 {
                            break;
                        }
                        k -= 1;

                        println!("{}", ws.word);
                    }
                }
                None => {
                    for line in lines {
                        match line {
                            Ok(line) if filter.accept(line.to_lowercase().as_str()) => {
                                out.write_all(line.as_bytes())?;
                                out.write_all(b"\n")?;
                            }
                            _ => continue,
                        }
                    }
                }
            }
        }
        Commands::Analyse {} => {
            let mut file = File::open(dict_path)?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;

            let mut char_freq = CharFreq::new();
            for c in buffer.chars() {
                char_freq.add_char(c);
            }

            let out = File::options()
                .create(true)
                .write(true)
                .open(char_freq_path)?;
            let mut out = BufWriter::new(out);
            for (c, count) in char_freq.to_vec() {
                out.write_all(format!("{}:{}\n", c, count).as_bytes())?;
            }
        }
    }

    Ok(())
}
