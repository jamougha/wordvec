 #[macro_use]
 #[macro_use]
extern crate clap;
extern crate yaml_rust;
extern crate time;

mod models;
mod parser;
mod mayberef;

use clap::{Arg, App};
use std::fs::{File, read_dir};
use std::io::{BufReader, BufRead, Read, Write, stdin};
use std::io;
use std::path::{Path};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use models::*;

fn get_line() -> String {
    let stdin = stdin();
    let mut buffer = String::new();
    stdin.read_line(&mut buffer).unwrap();
    buffer
}

fn find_most_common_words(corpus: &Path) -> Vec<(String, u32)> {
    let words = files(corpus).flat_map(|file| read_words(file));

    let mut word_counts = HashMap::new();
    for word in words {
        *word_counts.entry(word).or_insert(0) += 1;
    }

    let mut counts: Vec<_> = word_counts.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));

    counts
}

fn save_words(path: &Path, words: &Vec<(String, u32)>) -> io::Result<()> {
    let mut out = try!(File::create(path));
    for &(ref word, count) in words {
        try!(out.write_all(format!("{}, {}\n", &word, count).as_bytes()));
    }
    Ok(())
}

#[derive(Debug)]
struct FormatError;
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

fn load_most_common_words(filename: &str, num: usize) -> Result<Vec<(String, u32)>, Box<Error>> {
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

fn create_model(corpus: &Path, words: Vec<String>) -> LanguageModelBuilder {
    let start_time = time::get_time();
    let mut builder = LanguageModelBuilder::new(10, words);

    let mut num_words = 0;

    for sentence in files(corpus).flat_map(sentences) {
        let mut acc = builder.new_sentence();
        for word in sentence.split(|c| match c {
            'a'...'z' => false,
            _ => true,
        }).filter(|w| !w.is_empty())
        {
            num_words += 1;
            acc.add_word(word);
            if num_words % 10_000_000 == 0 {
                println!("Loaded {} million words in {} seconds",
                         num_words / 1_000_000,
                         time::get_time().sec - start_time.sec);
            }
        }
    }

    let end_time = time::get_time();
    println!("Model built in {}s", end_time.sec - start_time.sec);
    builder
}

fn main() {
    let matches = App::new("gauntlet")
        .version("0.0.1")
        .author("James Moughan <jamougha@gmail.com>")
        .about("Implementation of GloVe algorithm")
        .arg(Arg::with_name("CORPUS")
            .short("c")
            .long("corpus")
            .help("Sets a directory to search for the corpus")
            .takes_value(true))
        .arg(Arg::with_name("LOAD")
            .short("l")
            .long("load")
            .help("Loads a pre-saved language model")
            .takes_value(true))
        .arg(Arg::with_name("SAVE")
            .short("s")
            .long("save")
            .help("Generates a model from the corpus specified and saves it")
            .takes_value(true))
        .arg(Arg::with_name("WORDS")
            .short("w")
            .long("save_words")
            .help("Saves the vocabulary list to the specified file")
            .takes_value(true))
        .arg(Arg::with_name("LOAD_WORDS")
            .short("W")
            .long("load_words")
            .help("Loads the vocabulary list from the specified file")
            .takes_value(true))
        .arg(Arg::with_name("NUM_WORDS")
            .short("n")
            .long("num_words")
            .help("The maximum number of words to use in the vocabulary list, defaults to 30000")
            .takes_value(true))
        .get_matches();

    let (load, save, corpus) = (matches.value_of("LOAD"), matches.value_of("SAVE"), matches.value_of("CORPUS"));
    let builder = match (load, save, corpus) {
        (Some(l), None, None) => LanguageModelBuilder::load(Path::new(&l)).expect("Couldn't load model"),
        (Some(_), _, _) => { println!("You must specify either a model to load or a corpus directory location"); return; }
        (_, save, Some(corpus)) => {
            let corpus = Path::new(corpus);
            let words = matches.value_of("LOAD_WORDS").map(|file|
                load_most_common_words(file, 30000)
                   .unwrap_or_else(|e| panic!("Couldn't read vocabulary file: {}", e))
            ).unwrap_or(find_most_common_words(corpus));

            if let Some(wordfile) = matches.value_of("WORDS") {
                if let Err(e) = save_words(Path::new(wordfile), &words) {
                    println!("Couldn't save vocabulary list: {}", e);
                }
            }
            let builder = create_model(&corpus, words.into_iter().map(|x| x.0).collect());
            if let Some(save) = save {
                if let Err(e) = builder.save(Path::new(save)) {
                    println!("Couldn't save model: {}", e);
                }
            }
            builder
        }
        _ => panic!("what you want?")
    };

    let start_time = time::get_time();
    let model = builder.build();
    println!("Model built in {}s", time::get_time().sec - start_time.sec);

    loop {
        println!("");
        let input = get_line();
        if input.starts_with(":q") {
            break;
        }

        match parser::parse(input.trim_matches(|c: char| c.is_whitespace()), &model) {
            Ok(word_vec) => {
                let nearest = model.nearest_words(&word_vec);
                println!(" = {:?}", word_vec);
                println!("-------------");
                for word in nearest.iter().take(20) {
                    println!("{:?}, {}", word, word_vec.distance(word));
                }
            }
            Err(e) => println!("{:?}", e)
        }

    }

}
