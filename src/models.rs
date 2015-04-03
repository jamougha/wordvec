use std::ops::{Add, Sub};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::collections::VecDeque;
use std::cmp::Ordering::Equal;
use std::iter::repeat;
use std::fmt::{Debug, Formatter, Error};

struct WordVecBuilder {
    pub word: String,
    pub count: u64,
    vec: Vec<f32>,
}

pub struct WordVec {
    pub word: String,
    pub count: u64,
    vec: Vec<f32>,
}

impl Debug for WordVec {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        try!(self.word.fmt(fmt));
        try!(": ".fmt(fmt));
        self.count.fmt(fmt)
    }
}

impl WordVecBuilder {
    fn new(word: String, num_words: usize) -> WordVecBuilder {
        WordVecBuilder {
            word: word,
            count: 0,
            vec: repeat(0.0).take(num_words).collect(),
        }
    }

    fn normalize(mut self, word_freqs: &Vec<f32>) -> WordVec {
        let count = self.count as f32;
        for i in 0..word_freqs.len() {
            self.vec[i] = self.vec[i] / count /  word_freqs[i];
        }

        WordVec {
            word: self.word,
            count: self.count,
            vec: self.vec,
        }
    }

    fn inc(&mut self, i: usize) {
        self.vec[i] += 1.0;
        self.count += 1;
    }
}

impl WordVec {

    pub fn distance(&self, other: &WordVec) -> f32 {
        self.vec.iter()
                .zip(other.vec.iter())
                .map(|(x, y)| (x*x - y*y).abs().sqrt())
                .fold(0.0, |x, y| x + y)
    }

}

impl<'a> Add for &'a WordVec {
    type Output = WordVec;

    fn add(self, other: &WordVec) -> WordVec {
        debug_assert!(self.vec.len() == other.vec.len());
        let mut newvec = WordVec {
            word: format!("{} + {}", &self.word, &other.word),
            count: 0,
            vec: repeat(0.0).take(self.vec.len()).collect(),
        };

        for i in 0..10_000 {
            newvec.vec[i] = self.vec[i] + other.vec[i];
        }

        newvec
    }
}

impl<'a> Sub for &'a WordVec {
    type Output = WordVec;

    fn sub(self, other: &WordVec) -> WordVec {
        debug_assert!(self.vec.len() == other.vec.len());
        let mut newvec = WordVec {
            word: format!("{} + {}", &self.word, &other.word),
            count: 0,
            vec: repeat(0.0).take(self.vec.len()).collect(),
        };

        for i in 0..10_000 {
            newvec.vec[i] = self.vec[i] - other.vec[i];
        }

        newvec
    }
}

pub struct LanguageModel {
    words: HashMap<String, usize>,
    word_vecs: Vec<WordVec>,
}

pub struct LanguageModelBuilder {
    window_width: u32,
    words: HashMap<String, usize>,
    word_vecs: Vec<WordVecBuilder>,
}

pub struct WordAcceptor<'a> {
    window: VecDeque<String>,
    focus: usize,
    builder: &'a mut LanguageModelBuilder,
}

impl<'a> LanguageModelBuilder {
    pub fn new(words: Vec<String>) -> LanguageModelBuilder {

        let word_vecs = words.iter()
                             .map(|s| WordVecBuilder::new(s.clone(), words.len()))
                             .collect();

        let words = HashMap::from_iter(
                        words.into_iter()
                             .enumerate()
                             .map(|(a, b)| (b, a)));

        LanguageModelBuilder {
            window_width: 21,
            words: words,
            word_vecs: word_vecs,
        }
    }

    pub fn build(self) -> LanguageModel {
        let total_words = self.word_vecs.iter().fold(0.0, |a, b| a + b.count as f32);
        let word_freqs = self.word_vecs.iter()
                                       .map(|v| v.count as f32 / total_words)
                                       .collect::<Vec<_>>();
        LanguageModel {
            words: self.words,
            word_vecs: self.word_vecs.into_iter()
                                     .map(|b| b.normalize(&word_freqs))
                                     .collect(),
        }
    }

    pub fn new_file(&'a mut self) -> WordAcceptor<'a> {
        WordAcceptor {
            window: VecDeque::new(),
            focus: 0,
            builder: self,
        }
    }

    fn add_word(&mut self, word: &str) {
        let idx = self.words[word];
        let word_vec = &mut self.word_vecs[idx];
        word_vec.inc(idx);
    }
}

impl<'a> WordAcceptor<'a> {
    pub fn add_word(&mut self, word: String) {
        let allow_word = self.builder.words.contains_key(&word);

        if allow_word {
            self.window.push_back(word);
            if (self.window.len() > 21) {
                self.window.pop_front();
            }
            if (self.window.len() == 21) {
                for i in (0..21).filter(|a| *a != 10) {
                    self.builder.add_word(&self.window[i]);
                }
            }
        }
    }
}

impl LanguageModel {
    pub fn get(&self, word: &str) -> Option<&WordVec> {
        if let Some(i) = self.words.get(word) {
            Some(&self.word_vecs[*i])
        } else {
            None
        }
    }

    pub fn nearest_words(&self, word: &WordVec) -> Vec<&WordVec> {
        let mut vec_refs = self.word_vecs.iter().collect::<Vec<_>>();
        vec_refs.sort_by(|a, b| a.distance(word).partial_cmp(&b.distance(word)).unwrap_or(Equal));
        vec_refs
    }
}