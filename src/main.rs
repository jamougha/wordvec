#![feature(std_misc)]
mod models;

use std::fs::{File, read_dir};
use std::io::{BufReader, BufRead, Read, Write, stdout, copy};
use std::path::{Path, PathBuf};
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

    println!("{:?}", &counts[0..20]);
    println!("{:?}", &counts[counts.len() - 20..counts.len()]);
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
    const corpus_dir: &'static str = "/home/jamougha/corpus";
    //find_most_common_words(corpus_dir, "/home/jamougha/corpus/word_counts.csv");
    let words = load_most_common_words("/home/jamougha/corpus/word_counts.csv");
    let mut builder = LanguageModelBuilder::new(words);

    let path = Path::new(corpus_dir);

    visit_files(&path, |read| {
        let mut acc = (&mut builder).new_file();
        visit_tokens(read, |w| (&mut acc).add_word(w));
    });

    let model = builder.build();

    println!("{:?}", model.nearest_words(model.get("ubuntu").unwrap()));

}
