 #![feature(std_misc)]
 #![feature(str_words)]
 #![feature(scoped)]
extern crate time;

mod models;
mod parser;
mod mayberef;

use std::fs::{File, read_dir};
use std::io::{BufReader, BufRead, Read, Write, stdin};
use std::path::{Path};
use std::collections::HashMap;
use std::collections::hash_map::Entry::*;
use std::ascii::OwnedAsciiExt;
use models::*;
use std::thread::Builder;
use std::sync::mpsc::sync_channel;
use std::mem::swap;

fn get_line() -> String {
    let mut stdin = stdin();
    let mut buffer = String::new();
    stdin.read_line(&mut buffer).unwrap();
    buffer
}

fn find_most_common_words(corpus_loc: &str, outfile: &str) {
    let path = Path::new(corpus_loc);
    let mut word_counts = HashMap::new();
    let words = files(path).flat_map(|file| read_words(file));

    for word in words {
        match word_counts.entry(word) {
            Vacant(e) => { e.insert(1); },
            Occupied(mut e) => { *e.get_mut() += 1; },
        }
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
                        let lowercase = v.unwrap().into_ascii_lowercase();
                        String::from_utf8(lowercase).ok()
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
                           .map(|word| word.to_string().into_ascii_lowercase())
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

fn create_model(path: &Path) {
    const CORPUS_DIR: &'static str = "/home/jamougha/corpus/pg";
    const WORDS: &'static str = "/home/jamougha/corpus/pg/word_counts.csv";
    let start_time = time::get_time();
    find_most_common_words(CORPUS_DIR, WORDS);
    let words = load_most_common_words(WORDS, 30000);
    let mut builder = LanguageModelBuilder::new(10, words);

    let path = Path::new(CORPUS_DIR);

    let mut num_words = 0;

    for sentence in files(path).flat_map(sentences) {
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


    let model = builder.build();
    model.save(path);
    let end_time = time::get_time();
    println!("Model built in {}s", end_time.sec - start_time.sec);
}

fn main() {
    let path = Path::new("/home/jamougha/corpus/pg/model.data");
    let start_time = time::get_time();
    let model = LanguageModel::load(&path);
    let end_time = time::get_time();
    println!("Model loaded in {}s", start_time.sec - end_time.sec);

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
