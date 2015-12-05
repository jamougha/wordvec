 #![feature(std_misc)]
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
use std::path::{Path};
use std::collections::HashMap;
use std::collections::hash_map::Entry::*;
use models::*;

fn get_line() -> String {
    let mut stdin = stdin();
    let mut buffer = String::new();
    stdin.read_line(&mut buffer).unwrap();
    buffer
}

fn find_most_common_words(corpus: &Path, outfile: &str) {
    let words = files(corpus).flat_map(|file| read_words(file));

    let mut word_counts = HashMap::new();
    for word in words {
        *word_counts.entry(word).or_insert(0) += 1;
    }

    let mut counts: Vec<_> = word_counts.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));

    let mut out = File::create(&Path::new(outfile)).unwrap();
    for (word, count) in counts {
        out.write_all(format!("{}, {}\n", word, count).as_bytes()).unwrap();
    }

}

fn load_most_common_words(filename: &str, num: usize) -> Vec<String> {
    let file = File::open(&Path::new(filename)).unwrap();
    let reader = BufReader::new(file);
    reader.lines().map(|line|
        line.unwrap().split(',').next().unwrap().to_string()
    )
    .take(num)
    .collect()
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

fn create_model(corpus: &Path, model: &Path) -> LanguageModelBuilder {
    const WORDS: &'static str = "word_counts.csv";
    let start_time = time::get_time();
    find_most_common_words(corpus, WORDS);
    let words = load_most_common_words(WORDS, 30000);
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

    builder.save(model);
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
        .get_matches();

    let (load, save, corpus) = (matches.value_of("LOAD"), matches.value_of("SAVE"), matches.value_of("CORPUS"));
    let builder = match (load, save, corpus) {
        (Some(ref l), None, None) => LanguageModelBuilder::load(Path::new(&*l)),
        (Some(_), _, _) => { println!("You must specify either a model to load or a corpus directory location"); return; }
        (_, Some(ref s), Some(ref c)) => {
            let corpus = Path::new(c);
            let model = Path::new(s);
            create_model(&corpus, &model)
        }
        (_, _, Some(ref c)) => panic!("Couldn't find {:?}", c),
        _ => panic!("what")
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
