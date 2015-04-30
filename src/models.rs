use std::ops::{Add, Sub, Div};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::iter::repeat;
use std::fmt::{Debug, Formatter, Error};
use std::cmp::Ordering::Equal;
use std::ops::Drop;
use std::cmp;

#[derive(Clone,)]
pub struct WordVec {
    pub word: String,
    pub count: u64,
    vec: Vec<f32>,
}

impl PartialEq for WordVec {
    fn eq(&self, other: &WordVec) -> bool {
        self.vec == other.vec
    }
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

    fn dot_prod(&self, other: &WordVec) -> f32 {
        self.vec.iter()
                .zip(other.vec.iter())
                .map(|(x, y)| x * y)
                .fold(0.0, |x, y| x + y)
    }

    fn magnitude(&self) -> f32 {
        self.vec.iter()
                .map(|x| x * x)
                .fold(0.0, |x, y| x + y)
                .sqrt()
    }

    pub fn distance(&self, other: &WordVec) -> f32 {
        self.vec.iter().zip(other.vec.iter())
                .map(|(x, y)| (x - y)*(x - y))
                .fold(0.0, |x, y| x + y)
                .sqrt()
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

impl Div<i32> for WordVec {
    type Output = WordVec;

    fn div(mut self, i: i32) -> WordVec {
        for f in &mut self.vec {
            *f /= i as f32;
        }

        self
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
    window_radius: usize,
    words: HashMap<String, usize>,
    word_vecs: Vec<WordVec>,
}

pub struct WordAcceptor<'a> {
    sentence: Vec<Option<usize>>,
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
            window_radius: window_radius,
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

    pub fn new_sentence<'a>(&'a mut self) -> WordAcceptor<'a> {
        WordAcceptor {
            sentence: Vec::new(),
            builder: self,
        }
    }

    fn word_seen(&mut self, word: usize) {
        self.word_vecs[word].count += 1;
    }
}

impl<'a> WordAcceptor<'a> {
    pub fn add_word(&mut self, word: &str) {
        let idx_opt = self.builder.words.get(word).map(|w| *w);

        if let Some(next_idx) = idx_opt {
            self.builder.word_seen(next_idx);
            self.sentence.push(Some(next_idx));
        }
        else {
            self.sentence.push(None);
        }
    }
}

impl<'a> Drop for WordAcceptor<'a> {
    fn drop(&mut self) {
        let WordAcceptor {
            ref sentence,
            ref mut builder
        } = *self;

        for (i, &word_idx) in self.sentence.iter().enumerate() {
            let start = cmp::max(0, i as isize- builder.window_radius as isize) as usize;
            let end = cmp::min(sentence.len(), i + builder.window_radius + 1);
            for &j in &sentence[start..end] {
                match (word_idx, j) {
                    (Some(w), Some(j)) if w != j =>
                        builder.word_vecs[w].inc(j),
                    _ => {},
                }
            }
        }

        drop(sentence); // not actually sure if this is necessary
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
        vec_refs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or_else(|| {
            println!("bad vector: {:?}, {:?}", a, b);
            Equal
        }));
        vec_refs.into_iter().map(|x| x.1).collect()
    }
}

#[cfg(test)]
mod test {
    use super::{WordVec, LanguageModel, LanguageModelBuilder};

    #[test]
    fn test_accept_sentences() {
        let words = "foo bar baz blort".words().map(|w| w.to_string()).collect::<Vec<_>>();
        let mut builder = LanguageModelBuilder::new(1, words);

        let input = "x foo bar baz x x x x x x x blort".words();

        {
            let mut acc = builder.new_sentence();
            for word in input {
                acc.add_word(word);
            }
        }

        let model = builder.build();
        let foo = model.get("foo").unwrap();
        let baz = model.get("baz").unwrap();
        let bar = model.get("bar").unwrap();
        let blort = model.get("blort").unwrap();

        assert!(foo.distance(baz) < foo.distance(blort));
        assert!(foo.distance(bar) == bar.distance(baz));

    }
}