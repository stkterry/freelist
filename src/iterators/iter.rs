use std::marker::PhantomData;

use crate::Slot;

use super::size_hint;

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
            let curr = self.start;
            unsafe {
                self.start = self.start.offset(1);
                if let Slot::Value(value) = &*curr {
                    return Some(value)
                }
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::<T>(self.start as usize, self.end as usize)
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IterFl<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if self.end == self.start { return None }
            unsafe {
                self.end = self.end.offset(-1);
                if let Slot::Value(value) = &*self.end {
                    return Some(value)
                }
            }
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
    const ALL_SLICE: &[Slot<i32>; 3] = &[Slot::Value(0), Slot::Value(1), Slot::Value(2)];

    #[test]
    fn next() {
        let mut iter = IterFl::new(SLICE);

        assert_eq!(iter.next(), Some(&1)); 
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), None);

        let mut iter = IterFl::new(ALL_SLICE);
        for i in [0, 1, 2] { 
            assert_eq!(iter.next(), Some(&i));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn next_back() {
        let mut iter = IterFl::new(SLICE);

        assert_eq!(iter.next_back(), Some(&2)); 
        assert_eq!(iter.next_back(), Some(&1));
        assert_eq!(iter.next_back(), None); 

        let mut iter = IterFl::new(ALL_SLICE);
        for i in [2, 1, 0] { 
            assert_eq!(iter.next_back(), Some(&i));
        }
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