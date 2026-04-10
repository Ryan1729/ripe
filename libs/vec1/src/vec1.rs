use std::ops::{Deref, DerefMut, Index, IndexMut};

use std::num::NonZeroUsize;

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

    pub fn singleton_with_capacity(t: T, capacity: usize) -> Self {
        let mut v = Vec::with_capacity(capacity);
        v.push(t);

        Self(v)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(unused)]
    pub fn len1(&self) -> NonZeroUsize {
        self.0.len().try_into().expect("Invalid Vec1 was created!")
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

impl <T, const N: usize> TryFrom<[T; N]> for Vec1<T> {
    type Error = EmptyError;

    fn try_from(array: [T; N]) -> Result<Self, Self::Error> {
        Vec::from(array).try_into()
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
    };
    ($element: expr; $amount: expr) => {
        $crate::Vec1::try_from(vec![ $element; core::cmp::max(usize::from($amount), 1) ])
            .expect("vec1 macro should have syntactically prevented this error from happening!")
    };
}
pub use _vec1 as vec1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid1<Element, Width = NonZeroUsize> {
    pub width: Width,
    // TODO Since usize is u32 on wasm, let's make a Vec32 type that makes that restriction clear, so we
    // can't have like PC only grids that break in weird ways online. Probably no one will ever need that
    // many cells. Or maybe make it Vec1<Element, Index = usize>?
    pub cells: Vec1<Element>,
}

impl <Element, Width> Grid1<Element, Width> {
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    #[allow(unused)]
    pub fn len1(&self) -> NonZeroUsize {
        self.cells.len1()
    }

    pub fn get(&self, index: usize) -> Option<&Element> {
        self.cells.get(index)
    }

    #[allow(unused)]
    pub fn first(&self) -> &Element {
        self.cells.first()
    }

    #[allow(unused)]
    pub fn last(&self) -> &Element {
        self.cells.last()
    }
}

impl <Element, Width> Grid1<Element, Width> 
where Width: Clone {
    #[allow(unused)]
    pub fn map1<OldElement>(grid: &Grid1<OldElement, Width>, mapper: impl Fn(&OldElement) -> Element) -> Self {
        let width: Width = grid.width.clone();

        Self {
            width,
            cells: Vec1::map1(&grid.cells, mapper),
        }
    }

    pub fn slice(&self) -> (&[Element], Width) {
        (&self.cells, self.width.clone())
    }
}

impl<Element, Width, I: std::slice::SliceIndex<[Element]>> Index<I> for Grid1<Element, Width> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.cells[index]
    }
}

impl<Element, Width, I: std::slice::SliceIndex<[Element]>> IndexMut<I> for Grid1<Element, Width> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.cells[index]
    }
}

impl <'vec, Element, Width> Grid1<Element, Width> {
    fn iter(&'vec self) -> Iter<'vec, Element> {
        self.cells.iter()
    }
}

impl<'a, Element, Width> IntoIterator for &'a Grid1<Element, Width> {
    type Item = &'a Element;
    type IntoIter = Iter<'a, Element>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub struct Grid1Spec<Width> {
    pub width: Width,
    pub len: usize,
}

impl <Element, Width> Grid1<Element, Width> 
where Width: Clone {
    pub fn spec(&self) -> Grid1Spec<Width> {
        Grid1Spec {
            width: self.width.clone(),
            len: self.cells.len(),
        }
    }
}
