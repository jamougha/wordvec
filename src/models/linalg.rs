use models::LanguageModelBuilder;
use models::models::IndexableModel;
use std::ops::{IndexMut, Index, Mul};
use std::marker::PhantomData;

struct Matrix {
    height: usize,
    data: Vec<f32>,
}

impl Index<usize> for Matrix {
    type Output = [f32];

    fn index(&self, index: usize) -> &[f32] {
        let start = index * self.height;
        let end = start + self.height;
        &self.data[start..end]
    }
}


impl IndexMut<usize> for Matrix {
    fn index_mut(&mut self, index: usize) -> &mut [f32] {
        let start = index * self.height;
        let end = start + self.height;
        &mut self.data[start..end]
    }
}

fn dot(m: &[f32], n: &[f32]) -> f32 {
    m.iter()
     .zip(n.iter())
     .map(|(x, y)| x * y)
     .fold(0.0, |x, y| x + y)
}

pub fn solve(lmb: &LanguageModelBuilder, k: u32) {}
