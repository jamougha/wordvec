extern crate time;

use std::fs::{File, read_dir};
use std::io::{BufReader, BufRead, Read, Write};
use std::io;
use std::path::{Path};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use models::LanguageModelBuilder;


#[derive(Debug)]
pub struct FormatError;
impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid format for vocabulary file")
    }
}

impl Error for FormatError {
    fn description(&self) -> &str {
        "The format of the vocabulary file was Invalid"
    }
    fn cause(&self) -> Option<&Error> {
        None
    }
}

pub fn find_most_common_words(corpus: &Path) -> Vec<(String, u32)> {
    let words = files(corpus).flat_map(|file| read_words(file));

    let mut word_counts = HashMap::new();
    for word in words {
        *word_counts.entry(word).or_insert(0) += 1;
    }

    let mut counts: Vec<_> = word_counts.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));

    counts
}

pub fn save_words(path: &Path, words: &Vec<(String, u32)>) -> io::Result<()> {
    let mut out = try!(File::create(path));
    for &(ref word, count) in words {
        try!(out.write_all(format!("{}, {}\n", &word, count).as_bytes()));
    }
    Ok(())
}

pub fn load_most_common_words(filename: &str, num: usize) -> Result<Vec<(String, u32)>, Box<Error>> {
    let file = try!(File::open(&Path::new(filename)));
    let reader = BufReader::new(file);
    let mut words = vec!();
    for line in reader.lines().take(num) {
        if let Ok(line) = line {
            let mut columns = line.split(',');
            let word = columns.next().map(|w| w.to_string());
            let num = columns.next().map(|n| n.parse());
            match (word, num) {
                (Some(word), Some(num)) => words.push((word, try!(num))),
                _ => return Err(Box::new(FormatError)),
            }
        } else {
            return Err(Box::new(FormatError));
        }
    }
    Ok(words)
}

fn sentences<T: Read + 'static>(reader: BufReader<T>) -> Box<Iterator<Item = String>> {
    Box::new(reader.split('.' as u8)
                   .filter_map(|v| {
                        let lowercase = v.unwrap();
                        String::from_utf8(lowercase).ok().map(|s| s.to_lowercase())
                    }))
}

fn read_words<R: BufRead + 'static>(reader: R) -> Box<Iterator<Item = String>> {
    Box::new(reader.lines()
                   .filter_map(|line| line.ok())
                   .flat_map(|line| {
                       line.split(|c| match c {
                                'a'...'z' | 'A'...'Z' => false,
                                _ => true,
                            })
                           .filter(|word| !word.is_empty())
                           .map(|word| word.to_lowercase())
                           .collect::<Vec<_>>()
                           .into_iter()
                   }))
}

fn files(path: &Path) -> Box<Iterator<Item = BufReader<File>>> {
    Box::new(read_dir(path)
            .unwrap()
            .map(|path| path.unwrap().path())
            .filter(|path| path.to_str().unwrap().ends_with(".txt"))
            .map(|path| {
                let file = File::open(&path).unwrap();
                BufReader::new(file)
            }))
}

pub fn create_model(corpus: &Path, words: Vec<String>) -> LanguageModelBuilder {
    let mut builder = LanguageModelBuilder::new(10, words);

    for sentence in files(corpus).flat_map(sentences) {
        let mut acc = builder.new_sentence();
        for word in sentence.split(|c| match c {
            'a'...'z' => false,
            _ => true,
        }).filter(|w| !w.is_empty())
        {
            acc.add_word(word);
        }
    }

    builder
}