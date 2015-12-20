use rand::distributions::{IndependentSample, Range};

use std::iter::repeat;
use std::ops::{IndexMut, Index, Mul, Deref, DerefMut};
use std::mem;

#[derive(Clone, Debug)]
pub struct Matrix {
    height: usize,
    data: Vec<f32>,
}

impl Matrix {
    pub fn random(rows: usize, cols: usize, min: f32, max: f32) -> Matrix {
        let range = Range::new(min, max);
        let mut rng = ::rand::weak_rng();
        Matrix {
            height: cols,
            data: repeat(range.ind_sample(&mut rng)).take(rows * cols).collect(),
        }
    }
}

pub struct Row([f32]);

impl Deref for Row {
    type Target = [f32];

    #[inline]
    fn deref(&self) -> &[f32] {
        &self.0
    }
}

impl DerefMut for Row {
    #[inline]
    fn deref_mut(&mut self) -> &mut [f32] {
        &mut self.0
    }
}

impl Index<usize> for Matrix {
    type Output = Row;

    #[inline]
    fn index(&self, index: usize) -> &Row {
        let start = index * self.height;
        let end = start + self.height;
        unsafe { mem::transmute(&self.data[start..end]) }
    }
}


impl IndexMut<usize> for Matrix {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Row {
        let start = index * self.height;
        let end = start + self.height;
        unsafe { mem::transmute(&mut self.data[start..end]) }
    }
}

impl<'a> Mul for &'a Row {
    type Output = f32;
    #[inline]
    fn mul(self, other: &Row) -> f32 {
        dot(&self.0, &other.0)
    }
}

#[inline]
fn dot(m: &[f32], n: &[f32]) -> f32 {
    m.iter()
     .zip(n.iter())
     .map(|(x, y)| x * y)
     .fold(0.0, |x, y| x + y)
}

#[cfg(test)]
mod test {
    use models::linalg::Matrix;

    #[test]
    fn test_row_mult() {
        let mat = Matrix {
            height: 3,
            data: vec![1., 2., 3., 4., 5., 6.],
        };

        assert_eq!(14., &mat[0] * &mat[0]);
        assert_eq!(18. + 10. + 4., &mat[0] * &mat[1]);
    }
}
