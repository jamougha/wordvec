use std::ops::{Add, Sub};
use std::collections::HashMap;

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
    fn new(word: String) -> WordVec {
        WordVec {
            word: word,
            count: 0,
            vec: Box::new([0f32; 10_000]),
        }
    }

    fn normalize(mut self) -> WordVec {
        for elem in &mut self.vec[..] {
            *elem /= self.count as f32;
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

pub struct LanguageModelBuilder {
    words: HashMap<String, usize>,
    word_vecs: Vec<WordVecBuilder>,
}

impl LanguageModelBuilder {
    fn new(words: Vec<String>) {
        let mut builder = LanguageModelBuilder {
            words: HashMap::new(),
            word_vecs: Vec::new(),
        }
        assert_eq!(words.len, 10_000);


    }
}