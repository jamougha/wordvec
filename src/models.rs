use std::ops::{Add, Sub};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::collections::VecDeque;

struct WordVecBuilder {
    pub word: String,
    pub count: u64,
    vec: Box<[f32; 10_000]>,
}

pub struct WordVec {
    pub word: String,
    pub count: u64,
    vec: Box<[f32; 10_000]>,
}

impl WordVecBuilder {
    fn new(word: String) -> WordVecBuilder {
        WordVecBuilder {
            word: word,
            count: 0,
            vec: Box::new([0f32; 10_000]),
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
        let mut newvec = WordVec {
            word: format!("{} + {}", &self.word, &other.word),
            count: 0,
            vec: Box::new([0f32; 10_000]),
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
        let mut newvec = WordVec {
            word: format!("{} + {}", &self.word, &other.word),
            count: 0,
            vec: Box::new([0f32; 10_000]),
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
    words: HashMap<String, usize>,
    word_vecs: Vec<WordVecBuilder>,
}

pub struct WordAcceptor<'a> {
    window: VecDeque<String>,
    focus: usize,
    builder: &'a mut LanguageModelBuilder,
}

impl<'a> LanguageModelBuilder {
    fn new(words: Vec<String>) -> LanguageModelBuilder {
        assert_eq!(words.len(), 10_000);

        let word_vecs = words.iter()
                             .map(|s| WordVecBuilder::new(s.clone()))
                             .collect();

        let words = HashMap::from_iter(
                        words.into_iter()
                             .enumerate()
                             .map(|(a, b)| (b, a)));

        LanguageModelBuilder {
            words: words,
            word_vecs: word_vecs,
        }
    }

    fn build(self) -> LanguageModel {
        let total_words = self.word_vecs.iter().fold(0.0, |a, b| a + b.count as f32);
        let word_freqs = self.word_vecs.iter()
                                       .map(|v| v.count as f32 / total_words as f32)
                                       .collect::<Vec<_>>();
        LanguageModel {
            words: self.words,
            word_vecs: self.word_vecs.into_iter()
                                     .map(|b| b.normalize(&word_freqs))
                                     .collect(),
        }
    }

    fn newFile(&'a mut self) -> WordAcceptor<'a> {
        WordAcceptor {
            window: VecDeque::new(),
            focus: 0,
            builder: self,
        }
    }
}

impl<'a> WordAcceptor<'a> {
    fn word(&mut self, word: String) {
        if let Some(idx) = self.builder.words.get(&word) {
            self.window.push_back(word);
            if (self.window.len() > 21) {
                self.window.pop_front();
            }
        }
    }
}