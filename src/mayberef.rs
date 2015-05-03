use std::ops::Deref;
use self::MaybeRef::*;

pub enum MaybeRef<'a, T: 'a> {
    Ref(&'a T),
    Val(T),
}

impl<'a, T: Clone> MaybeRef<'a, T> {
    pub fn take(self) -> T {
        match self {
            Val(t) => t,
            Ref(t) => t.clone(),
        }
    }
}

impl<'a, T> Deref for MaybeRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match *self {
            Ref(t) => t,
            Val(ref t) => t,
        }
    }
}

#[cfg(test)]
mod test {
    use super::MaybeRef::*;

    #[test]
    fn test() {
        let x = "foo";
        let r = Ref(&x);
        let v = Val(x);
        assert_eq!(r.take(), v.take());
    }
}
