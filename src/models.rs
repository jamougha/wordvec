use std::ops::{Add, Sub};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::collections::VecDeque;
use std::cmp::Ordering::Equal;
use std::iter::repeat;
use std::fmt::{Debug, Formatter, Error};

#[derive(Clone)]
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

impl WordVec {

    pub fn distance(&self, other: &WordVec) -> f32 {
        self.vec.iter()
                .zip(other.vec.iter())
                .map(|(x, y)| (x*x - y*y).abs().sqrt())
                .fold(0.0, |x, y| x + y)
    }

    fn new(word: String, num_words: usize) -> WordVec {
        WordVec {
            word: word,
            count: 0,
            vec: repeat(0.0).take(num_words).collect(),
        }
    }

    fn normalize(&mut self, word_freqs: &Vec<f32>) {
        let count = self.count as f32;
        for i in 0..word_freqs.len() {
            self.vec[i] = self.vec[i] / count;
        }
    }

    fn inc(&mut self, i: usize) {
        self.vec[i] += 1.0;
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
    window_width: usize,
    words: HashMap<String, usize>,
    word_vecs: Vec<WordVec>,
}

pub struct WordAcceptor<'a> {
    window: VecDeque<String>,
    builder: &'a mut LanguageModelBuilder,
}

impl<'a> LanguageModelBuilder {
    pub fn new(words: Vec<String>) -> LanguageModelBuilder {

        let word_vecs = words.iter()
                             .map(|s| WordVec::new(s.clone(), words.len()))
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

    pub fn build(mut self) -> LanguageModel {
        let total_words = self.word_vecs.iter().fold(0.0, |a, b| a + b.count as f32);
        let word_freqs = self.word_vecs.iter()
                                       .map(|v| v.count as f32 / total_words)
                                       .collect::<Vec<_>>();
        for vec in self.word_vecs.iter_mut() {
            vec.normalize(&word_freqs);
        }

        LanguageModel {
            words: self.words,
            word_vecs: self.word_vecs,
        }
    }

    pub fn new_file(&'a mut self) -> WordAcceptor<'a> {
        WordAcceptor {
            window: VecDeque::new(),
            builder: self,
        }
    }

    fn get_vec(&mut self, word: &str) -> &mut WordVec {
        let i = self.words[word];
        &mut self.word_vecs[i]
    }

    fn add_word(&mut self, from: &str, to: &str) {
        let to_idx = self.words[to];
        let from_vec = self.get_vec(from);
        from_vec.inc(to_idx);
    }

    fn word_seen(&mut self, word: &str) {
        self.get_vec(word).count += 1;
    }
}

impl<'a> WordAcceptor<'a> {
    pub fn add_word(&mut self, word: String) {
        let ww = self.builder.window_width;
        let allow_word = self.builder.words.contains_key(&word);

        if allow_word {
            self.window.push_back(word);
            if self.window.len() > ww {
                self.window.pop_front();
            }
            if self.window.len() == ww {
                self.builder.word_seen(&self.window[ww / 2]);
                for i in (0..ww).filter(|a| *a != (ww / 2)) {
                    self.builder.add_word(&self.window[ww / 2], &self.window[i]);
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
        let mut vec_refs = self.word_vecs.iter().filter_map(|w|
            if w.word != word.word { Some((w.distance(word), w)) } else { None }
        ).collect::<Vec<_>>();
        vec_refs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Equal));
        vec_refs.into_iter().map(|x| x.1).collect()
    }
}