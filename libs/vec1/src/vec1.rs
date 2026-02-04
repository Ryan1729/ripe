use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Debug, PartialEq, Eq)]
pub struct Vec1<T>(Vec<T>);

impl<T: Clone> Clone for Vec1<T> {
    #[track_caller]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Default> Default for Vec1<T> {
    #[track_caller]
    fn default() -> Self {
        Self(vec![T::default()])
    }
}

impl <T> Vec1<T> {
    pub fn singleton(t: T) -> Self {
        vec1![t]
    }

    // TODO? A len1 that returns a NonZeroUsize?
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, value: T) {
        self.0.push(value)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    pub fn first(&self) -> &T {
        self.0.first().expect("Invalid Vec1 was created!")
    }

    pub fn last(&self) -> &T {
        self.0.last().expect("Invalid Vec1 was created!")
    }

    pub fn map1<U>(vec: &Vec1<U>, mapper: impl Fn(&U) -> T) -> Self {
        let mut output = Vec::with_capacity(vec.len());

        for element in vec.iter() {
            output.push(mapper(element));
        }

        Vec1::try_from(output).expect("The input being a Vec1 should prevent this case!")
    }
}

impl<T, I: std::slice::SliceIndex<[T]>> Index<I> for Vec1<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.0[index]
    }
}

impl<T, I: std::slice::SliceIndex<[T]>> IndexMut<I> for Vec1<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T> Deref for Vec1<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self[..]
    }
}

impl<T> DerefMut for Vec1<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self[..]
    }
}

type Iter<'a, T> = std::slice::Iter<'a, T>;

impl <'vec, T> Vec1<T> {
    fn iter(&'vec self) -> Iter<'vec, T> {
        self.0.iter()
    }
}

impl<'a, T> IntoIterator for &'a Vec1<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub struct EmptyError;

impl <T> TryFrom<Vec<T>> for Vec1<T> {
    type Error = EmptyError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(EmptyError)
        } else {
            Ok(Vec1(value))
        }
    }
}

impl <T> From<Vec1<T>> for Vec<T> {
    fn from(value: Vec1<T>) -> Self {
        value.0
    }
}

#[macro_export]
macro_rules! _vec1 {
    ($($element: expr),+ $(,)?) => {
        $crate::Vec1::try_from(vec![ $($element),+ ])
            .expect("vec1 macro should have syntactically prevented this error from happening!")
    }
}
pub use _vec1 as vec1;