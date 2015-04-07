#![feature(std_misc)]
mod models;
mod parser;
mod mayberef;

use std::fs::{File, read_dir};
use std::io::{BufReader, BufRead, Read, Write};
use std::path::{Path};
use std::collections::HashMap;
use std::collections::hash_map::Entry::*;
use std::ascii::OwnedAsciiExt;
use models::*;

fn visit_tokens<T: Read, F: FnMut(String) -> ()>(reader: BufReader<T>, mut counter: F) {
    for line in reader.lines() {
    	if let Ok(line) = line {
	    	for word in line.split(|c: char| !c.is_alphabetic()) {
	    		if word.len() > 0 {
	    			let word = word.to_string().into_ascii_lowercase();
		    		counter(word);
		    	}
	    	}
	    }
    }
}

fn find_most_common_words(corpus_loc: &str, outfile: &str) {
    let path = Path::new(corpus_loc);
    let mut word_counts = HashMap::new();
    visit_files(path, &mut |read| visit_tokens(read, &mut |s|
        match (&mut word_counts).entry(s) {
            Vacant(e) => { e.insert(1); },
            Occupied(mut e) => { *e.get_mut() += 1; },
    }));

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
    ).collect()
}

fn visit_files<F: FnMut(BufReader<File>) -> ()>(path: &Path, mut file_processor: F) {
    let files = read_dir(path);
    for file in files.unwrap() {
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
    let mut builder = LanguageModelBuilder::new(words);

    let path = Path::new(CORPUS_DIR);

    visit_files(&path, |read| {
        let mut acc = (&mut builder).new_file();
        visit_tokens(read, |w| (&mut acc).add_word(w));
    });

    let model = builder.build();

    let linux = model.get("king").unwrap();
    let nearest = model.nearest_words(linux);
    for word in nearest.iter().take(20) {
        println!("{:?}, {}", word, linux.distance(word));
    }

}
