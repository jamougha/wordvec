use std::ops::{Add, Sub};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::collections::VecDeque;
use std::iter::repeat;
use std::fmt::{Debug, Formatter, Error};

#[derive(Clone, PartialEq)]
pub struct WordVec {
    pub word: String,
    pub count: u64,
    vec: Vec<f32>,
}

impl Debug for WordVec {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        try!(self.word.fmt(fmt));
        try!(fmt.write_str(": "));
        try!(self.count.fmt(fmt));
        try!(fmt.write_str(", "));
        self.vec.iter().take(5).collect::<Vec<_>>().fmt(fmt)
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

    fn normalize(&mut self) {
        let count = self.count as f32;
        for f in &mut self.vec {
            *f /= count;
        }
    }

    fn inc(&mut self, i: usize) {
        self.vec[i] += 1.0;
    }

}

impl<'a> Add<&'a WordVec> for WordVec {
    type Output = WordVec;

    fn add(self, other: &WordVec) -> WordVec {
        debug_assert!(self.vec.len() == other.vec.len());
        let mut newvec = WordVec {
            word: format!("{} + {}", &self.word, &other.word),
            count: 0,
            vec: self.vec,
        };

        for i in 0..newvec.vec.len() {
            newvec.vec[i] += other.vec[i];
        }

        newvec
    }
}

impl<'a> Sub<&'a WordVec> for WordVec {
    type Output = WordVec;

    fn sub(self, other: &WordVec) -> WordVec {
        debug_assert!(self.vec.len() == other.vec.len());
        let mut newvec = WordVec {
            word: format!("{} - {}", &self.word, &other.word),
            count: 0,
            vec: self.vec,
        };

        for i in 0..(newvec.vec.len()) {
            newvec.vec[i] -= other.vec[i];
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
    window: VecDeque<(usize, String)>,
    builder: &'a mut LanguageModelBuilder,
}

impl LanguageModelBuilder {
    pub fn new(window_radius: usize, words: Vec<String>) -> LanguageModelBuilder {

        let word_vecs = words.iter()
                             .map(|s| WordVec::new(s.clone(), words.len()))
                             .collect();

        let words = HashMap::from_iter(
                        words.into_iter()
                             .enumerate()
                             .map(|(a, b)| (b, a)));

        LanguageModelBuilder {
            window_width: 2 * window_radius + 1,
            words: words,
            word_vecs: word_vecs,
        }
    }

    pub fn build(mut self) -> LanguageModel {
        for vec in self.word_vecs.iter_mut() {
            vec.normalize();
        }

        LanguageModel {
            words: self.words,
            word_vecs: self.word_vecs,
        }
    }

    pub fn new_file<'a>(&'a mut self) -> WordAcceptor<'a> {
        WordAcceptor {
            window: VecDeque::new(),
            builder: self,
        }
    }

    fn get_vec(&mut self, word: &str) -> &mut WordVec {
        let i = self.words[word];
        &mut self.word_vecs[i]
    }

    fn add_word(&mut self, from: usize, to: usize) {
        self.word_vecs[from].inc(to);
    }

    fn word_seen(&mut self, word: &str) {
        self.get_vec(word).count += 1;
    }
}

impl<'a> WordAcceptor<'a> {
    pub fn add_word(&mut self, word: String) {
        let ww = self.builder.window_width;
        let idx_opt = self.builder.words.get(&word).map(|w| *w);

        if let Some(next_idx) = idx_opt {
            self.window.push_back((next_idx, word));
            if self.window.len() > ww {
                self.window.pop_front();
            }

            if self.window.len() == ww {
                let (center_idx, ref center_word) = self.window[ww / 2];
                self.builder.word_seen(center_word);
                for i in (0..ww).filter(|a| *a != ww / 2) {
                    self.builder.add_word(center_idx, self.window[i].0);
                }
            }
        }
    }
}

impl LanguageModel {
    pub fn get(&self, word: &str) -> Option<&WordVec> {
        self.words.get(word).map(|i| &self.word_vecs[*i])
    }

    pub fn nearest_words(&self, word: &WordVec) -> Vec<&WordVec> {
        let mut vec_refs = self.word_vecs.iter().filter_map(|w| {
            let dist = w.distance(word);
            if w.word != word.word {
                Some((dist, w))
            } else {
                None
            }
        }).collect::<Vec<_>>();
        vec_refs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or_else(||
            panic!("{:?}, {:?}", a, b)));
        vec_refs.into_iter().map(|x| x.1).collect()
    }
}