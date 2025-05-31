use std::marker::PhantomData;

use crate::Slot;

pub struct IterMutFl<'a, T: 'a> {
    start: *mut Slot<T>,
    end: *mut Slot<T>,
    _marker: PhantomData<&'a mut T>
}


impl<'a, T: 'a> IterMutFl<'a, T> {

    #[inline]
    pub(crate) const fn new(slice: &mut [Slot<T>]) -> Self {
        let start = slice.as_mut_ptr();
        Self {
            start,
            end: match slice.len() {
                0 => start,
                count @ _ => unsafe { start.add(count) }
            },
            _marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for IterMutFl<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.start == self.end { return None }
            unsafe { match &mut *self.start {
                Slot::Value(value) => {
                    self.start = self.start.offset(1);
                    return Some(value)
                },
                _ => { self.start = self.start.offset(1) }
            }}
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) 
            / std::mem::size_of::<Slot<T>>();
        (len, Some(len))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IterMutFl<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if self.end == self.start { return None }
            unsafe { match &mut *self.end {
                Slot::Value(value) => {
                    self.end = self.end.offset(-1);
                    return Some(value) 
                },
                _ => { self.end = self.end.offset(-1) }
            }}
        }
    }
}

impl<'a, T: 'a> Drop for IterMutFl<'a, T> {
    fn drop(&mut self) { for _ in &mut * self { } }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next() {
        let slice = &mut [Slot::Empty, Slot::Value(1), Slot::Next(0), Slot::Value(2)];
        let mut iter = IterMutFl::new(slice);

        assert_eq!(iter.next(), Some(&mut 1)); 
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), None); 
    }

        #[test]
    fn next_back() {
        let slice = &mut [Slot::Empty, Slot::Value(1), Slot::Next(0), Slot::Value(2)];
        let mut iter = IterMutFl::new(slice);

        assert_eq!(iter.next_back(), Some(&mut 2)); 
        assert_eq!(iter.next_back(), Some(&mut 1));
        assert_eq!(iter.next_back(), None); 
    }

    #[test]
    fn size_hint() {
        let slice = &mut [Slot::Empty, Slot::Value(1), Slot::Next(0), Slot::Value(2)];
        let mut iter = IterMutFl::new(slice);
        assert_eq!(iter.size_hint(), (4, Some(4)));
        iter.next();
        assert_eq!(iter.size_hint(), (2, Some(2)));
    }

    #[test]
    fn update_value() {
        let slice = &mut [Slot::Empty, Slot::Value(1), Slot::Next(0), Slot::Value(2)];
        let mut iter = IterMutFl::new(slice);
        *iter.next().unwrap() = 11;
        assert_eq!(slice[1], Slot::Value(11));
    }
}