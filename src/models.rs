use std::ops::{Add, Sub, Div};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::iter::repeat;
use std::fmt::{Debug, Formatter, Error};
use std::cmp::Ordering::Equal;
use std::ops::Drop;
use std::cmp;
use std::path::Path;
use std::io::{BufWriter, Write, BufReader, Read, BufRead};
use std::fs::File;
use std::mem;

#[derive(Clone)]
pub struct WordVec {
    pub word: String,
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
            vec: repeat(0.0).take(num_words).collect(),
        }
    }

    fn normalize(&mut self) {
        let cooccurences = self.vec.iter().fold(0f32, |x, y| {
            assert!(!y.is_nan());
            x + *y
        });

        for f in &mut self.vec {
            *f /= cooccurences + 1.0;
        }
    }

    fn inc(&mut self, i: usize, dist: usize) {
        assert!(dist != 0);
        self.vec[i] += 1.0 / dist as f32;
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
            vec: self.vec,
        };

        for i in 0..(newvec.vec.len()) {
            newvec.vec[i] -= other.vec[i];
        }

        newvec
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct LanguageModel {
    words: HashMap<String, usize>,
    word_vecs: Vec<WordVec>,
}

pub struct LanguageModelBuilder {
    window_radius: usize,
    words: HashMap<String, usize>,
    word_vecs: Vec<WordVec>,
    sentence: Vec<Option<usize>>,
}

pub struct WordAcceptor<'a> {
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
            sentence: vec![],
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
            builder: self,
        }
    }

    pub fn save(&self, path: &Path) {
        let mut file = BufWriter::new(File::create(path).unwrap());

        write_raw(self.word_vecs.len(), &mut file);
        file.write(&[b'\n']).unwrap();

        for vec in &self.word_vecs {
            file.write(vec.word.as_bytes()).unwrap();
            file.write(&[b':']).unwrap();
            for f in &vec.vec {
                write_raw(*f, &mut file)
            }
            file.write(&[b'\n']).unwrap();
        }

    }

    pub fn load(path: &Path) -> LanguageModelBuilder {
        let mut file = BufReader::new(File::open(path).unwrap());
        let mut word_vecs = Vec::new();

        let size: u64 = unsafe { read_raw::<u64, _>(&mut file) };
        read_byte(b'\n', &mut file);

        for _ in 0..size {
            let mut word: Vec<u8> = Vec::new();
            file.read_until(b':', &mut word).unwrap();
            assert_eq!(Some(b':'), word.pop());
            let word = String::from_utf8(word).unwrap();

            let mut vec: Vec<f32> = repeat(0f32).take(size as usize).collect();
            for f in &mut vec {
                unsafe {
                    *f = read_raw::<f32, _>(&mut file);
                }
            }

            read_byte(b'\n', &mut file);

            word_vecs.push(WordVec {
                word: word,
                vec: vec,
            });
        }

        let words: HashMap<_, _> = word_vecs.iter()
                                            .map(|v| v.word.clone())
                                            .enumerate()
                                            .map(|(i, w)| (w, i))
                                            .collect();

        LanguageModelBuilder {
            window_radius: 0,
            words: words,
            word_vecs: word_vecs,
            sentence: Vec::new(),
        }
    }
}

impl<'a> WordAcceptor<'a> {
    pub fn add_word(&mut self, word: &str) {
        let idx_opt = self.builder.words.get(word).map(|w| *w);

        self.builder.sentence.push(idx_opt);
    }
}

impl<'a> Drop for WordAcceptor<'a> {
    fn drop(&mut self) {
        let LanguageModelBuilder {
            ref mut sentence,
            ref mut word_vecs,
            window_radius,
            ..
        } = *self.builder;

        for (i, &from_word) in sentence.iter().enumerate() {
            let start = cmp::max(0, i as isize - window_radius as isize) as usize;
            let end = cmp::min(sentence.len(), i + window_radius + 1);
            for j in (start..i).chain(i + 1..end) {
                let dist = cmp::max(i, j) - cmp::min(i, j);

                if let (Some(f), Some(t)) = (from_word, sentence[j]) {
                    word_vecs[f].inc(t, dist);
                }
            }
        }

        sentence.clear();
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

fn read_byte<R: Read>(b: u8, read: &mut R) {
    let buf = &mut [0];
    while read.read(buf).unwrap() == 0 {}
    assert_eq!(&[b], buf);
}

unsafe fn read_raw<T: Copy, R: Read>(reader: &mut BufReader<R>) -> T {
    let mut buffer = [0u8; 64];
    let t_size = mem::size_of::<T>();
    assert!(t_size <= buffer.len());

    let mut remainder = t_size;
    while remainder > 0 {
        let bytes_read = reader.read(&mut buffer[(t_size-remainder)..t_size]).unwrap();
        remainder -= bytes_read;
    }

    let bptr: *mut T = mem::transmute(buffer.as_ptr());
    *bptr
}

fn write_raw<T: Copy, F: Write>(t: T, writer: &mut BufWriter<F>) {
    let mut buffer = [0u8; 64];
    let t_size = mem::size_of::<T>();
    assert!(t_size <= buffer.len());
    unsafe {
        let bptr: *mut T = mem::transmute(buffer.as_ptr());
        *bptr = t;
    }

    writer.write(&buffer[0..t_size]);
}

#[cfg(test)]
mod test {
    use super::{WordVec, LanguageModel, LanguageModelBuilder};
    use std::path::Path;

    fn get_builder() -> LanguageModelBuilder {
        let words = "foo bar baz blort".split(" ").map(|w| w.to_string()).collect::<Vec<_>>();
        let mut builder = LanguageModelBuilder::new(1, words);

        let input = "x foo bar baz x x x x x x x blort".split(" ");

        {
            let mut acc = builder.new_sentence();
            for word in input {
                acc.add_word(word);
            }
        }

        builder
    }

    #[test]
    fn test_accept_sentences() {
        let model = get_builder().build();
        let foo = model.get("foo").unwrap();
        let baz = model.get("baz").unwrap();
        let bar = model.get("bar").unwrap();
        let blort = model.get("blort").unwrap();

        assert!(foo.distance(baz) < foo.distance(blort));
        assert!(foo.distance(bar) == bar.distance(baz));
    }

    #[test]
    fn test_serialization() {
        let builder = get_builder();
        let path = Path::new("/tmp/model.data");
        builder.save(&path);

        let loaded_model = LanguageModelBuilder::load(path).build();
        assert_eq!(builder.build(), loaded_model);
    }
}
