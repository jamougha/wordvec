#![feature(std_misc)]
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
    for &(ref word, count) in &counts[..] {
        out.write_all(format!("{}, {}\n", word, count).as_bytes()).unwrap();
    }

}

fn load_most_common_words(filename: &str) -> Vec<String> {
    let file = File::open(&Path::new(filename)).unwrap();
    let reader = BufReader::new(file);
    reader.lines().map(|line|
        line.unwrap().split(',').next().unwrap().to_string()
    )
    .take(num)
    .collect()
}

fn read_words<R: BufRead + 'static>(reader: R) -> Box<Iterator<Item=String>> {
    Box::new(reader.lines()
                   .filter_map(|line| line.ok())
                   .flat_map(|line| {
                       let words = line.split(|c: char| !c.is_alphabetic() && c != '\'')
                                       .map(|word| word.to_string())
                                       .collect::<Vec<_>>();
                       words.into_iter()
                   }))
}

fn files(path: &Path) -> Box<Iterator<Item=BufReader<File>>> {
    Box::new(
        read_dir(path)
            .unwrap()
            .map(|path| path.unwrap().path())
            .filter(|path| path.to_str().unwrap().ends_with(".txt"))
            .map(|path| {
                let file = File::open(&path).unwrap();
                BufReader::new(file)
            }))
}

fn visit_files<F: FnMut(BufReader<File>) -> ()>(path: &Path, num: usize, mut file_processor: F) {
    let files = read_dir(path);
    for file in files.unwrap().take(num) {
        let path = file.unwrap().path();
        if path.to_str().unwrap().ends_with(".txt") {
            let file = File::open(&path).unwrap();
            let reader = BufReader::new(file);
            file_processor(reader);
        }
    }
}

fn main() {
    const CORPUS_DIR: &'static str = "/home/jamougha/corpus/pg";
    const WORDS: &'static str = "/home/jamougha/corpus/pg/word_counts.csv";
    //find_most_common_words(CORPUS_DIR, WORDS);
    let words = load_most_common_words(WORDS);
    let mut builder = LanguageModelBuilder::new(10, words);

    let path = Path::new(CORPUS_DIR);

    for file in files(path) {
        let mut acc = builder.new_file();
        for word in read_words(file) {
            acc.add_word(word);
        }
    }

    let model = builder.build();
    println!("Model loaded");
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
