
#[macro_use]
#[macro_use]
extern crate clap;
extern crate time;

mod models;
mod parser;
mod mayberef;
mod processing;


use clap::{Arg, App};
use std::io::{BufRead, Read, stdin};
use std::path::Path;
use models::LanguageModelBuilder;
use processing::{find_most_common_words, save_words, load_most_common_words, create_model};

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
                               .help("The maximum number of words to use in the vocabulary \
                                      list, defaults to 30000")
                               .takes_value(true))
                      .get_matches();

    let (load, corpus) = (matches.value_of("LOAD"), matches.value_of("CORPUS"));
    let builder = match (load, corpus) {
        (Some(l), None) => LanguageModelBuilder::load(Path::new(&l)).expect("Couldn't load model"),
        (None, Some(corpus)) => {
            let corpus = Path::new(corpus);
            let num_words = matches.value_of("NUM_WORDS")
                                   .map(|n| n.parse().expect("Number of words was invalid"))
                                   .unwrap_or(30000);
            let words = matches.value_of("LOAD_WORDS")
                               .map(|file| {
                                   load_most_common_words(file, num_words).unwrap_or_else(|e| {
                                       panic!("Couldn't read vocabulary file: {}", e)
                                   })
                               })
                               .unwrap_or(find_most_common_words(corpus, num_words));
            panic!("foo");
            if let Some(wordfile) = matches.value_of("WORDS") {
                if let Err(e) = save_words(Path::new(wordfile), &words) {
                    println!("Couldn't save vocabulary list: {}", e);
                }
            }
            let builder = create_model(&corpus, words.into_iter().map(|x| x.0).collect());
            if let Some(save) = matches.value_of("SAVE") {
                if let Err(e) = builder.save(Path::new(save)) {
                    println!("Couldn't save model: {}", e);
                }
            }

            builder
        }
        _ => {
            println!("You must specify either a model to load or a corpus directory location");
            return;
        }
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
            Err(e) => println!("{:?}", e),
        }

    }

}

fn get_line() -> String {
    let stdin = stdin();
    let mut buffer = String::new();
    stdin.read_line(&mut buffer).unwrap();
    buffer
}
