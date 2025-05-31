use std::marker::PhantomData;

use crate::Slot;

pub struct IterFl<'a, T: 'a> {
    start: *const Slot<T>,
    end: *const Slot<T>,
    _marker: PhantomData<&'a T>
}


impl<'a, T: 'a> IterFl<'a, T> {

    #[inline]
    pub(crate) const fn new(slice: &[Slot<T>]) -> Self {
        let start = slice.as_ptr();
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

impl<'a, T: 'a> Iterator for IterFl<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.start == self.end { return None }
            unsafe { match &*self.start {
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

impl<'a, T: 'a> DoubleEndedIterator for IterFl<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if self.end == self.start { return None }
            unsafe { match &*self.end {
                Slot::Value(value) => {
                    self.end = self.end.offset(-1);
                    return Some(value) 
                },
                _ => { self.end = self.end.offset(-1) }
            }}
        }
    }
}

impl<'a, T: 'a> Drop for IterFl<'a, T> {
    fn drop(&mut self) { for _ in &mut * self { } }
}


#[cfg(test)]
mod tests {
    use super::*;
    static SLICE: &[Slot<i32>] = &[Slot::Empty, Slot::Value(1), Slot::Next(0), Slot::Value(2)];

    #[test]
    fn next() {
        let mut iter = IterFl::new(SLICE);

        assert_eq!(iter.next(), Some(&1)); 
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), None); 
    }

        #[test]
    fn next_back() {
        let mut iter = IterFl::new(SLICE);

        assert_eq!(iter.next_back(), Some(&2)); 
        assert_eq!(iter.next_back(), Some(&1));
        assert_eq!(iter.next_back(), None); 
    }

    #[test]
    fn size_hint() {
        let mut iter = IterFl::new(SLICE);

        assert_eq!(iter.size_hint(), (4, Some(4)));
        iter.next();
        assert_eq!(iter.size_hint(), (2, Some(2)));
    }
}