#![feature(std_misc)]
mod models;

use std::fs::{File, read_dir};
use std::io::{BufReader, BufRead, Read, Write};
use std::path::Path;
use std::collections::HashMap;
use std::collections::hash_map::Entry::*;
use std::ascii::OwnedAsciiExt;
use models::*;

fn count_tokens<T: Read>(reader: BufReader<T>, word_counts: &mut HashMap<String, u32>) {

    for line in reader.lines().skip(30) {
    	if let Ok(line) = line {
	    	for word in line.split(|c: char| !c.is_alphabetic()) {
	    		if word.len() > 0 {
	    			let word = word.to_string().into_ascii_lowercase();
		    		match word_counts.entry(word) {
		    			Occupied(mut e) => { *e.get_mut() += 1; },
		    			Vacant(e) => { e.insert(1); }
		    		}
		    	}
	    	}
	    }
    }
}

fn find_most_common_words(corpus_loc: &str, outfile: &str) {
    let path = Path::new(corpus_loc);
    let files = read_dir(path);
    let mut word_counts = HashMap::new();

    for file in files.unwrap() {
    	let path = file.unwrap().path();
    	if path.to_str().unwrap().ends_with(".txt") {
		    let file = File::open(&path).unwrap();
		    let reader = BufReader::new(file);
    		count_tokens(reader, &mut word_counts);
    		
    	}
	    println!("{:?}", word_counts.len());
    }

    let mut counts: Vec<_> = word_counts.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));

    let mut out = File::create(&Path::new(outfile)).unwrap();
    for &(ref word, count) in &counts[0..10000] {
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


fn main() {
    // find_most_common_words("/home/jamougha/corpus/pg", "/home/jamougha/corpus/pg/word_counts.csv");
    let words = load_most_common_words("/home/jamougha/corpus/pg/word_counts.csv");
    println!("{:?}", words);

}
