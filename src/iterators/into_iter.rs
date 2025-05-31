
use crate::{Freelist, Slot};

use super::size_hint;

pub struct IntoIterFl<T> {
    start: *const Slot<T>,
    end: *const Slot<T>,
    _fl: Freelist<T>
}

impl<T> IntoIterFl<T> {
    #[inline]
    pub(crate) fn new(freelist: Freelist<T>) -> Self {
        let start = freelist.slots.as_ptr();
        Self {
            start,
            end: match freelist.slots.len() {
                0 => start,
                count @ _ => unsafe { start.add(count) }
            },
            _fl: freelist
        }
    }
}

impl<T> Iterator for IntoIterFl<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.start == self.end { return None }
            let curr = self.start;
            unsafe {
                self.start = self.start.offset(1);
                if let Slot::Value(value) = curr.read() {
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

impl<T> DoubleEndedIterator for IntoIterFl<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if self.start == self.end { return None }
            unsafe {
                self.end = self.end.offset(-1);
                if let Slot::Value(value) = self.end.read() {
                    return Some(value)
                }
            }
        }
    }
}

impl<T> Drop for IntoIterFl<T> {
    fn drop(&mut self) { for _ in &mut * self { } }
}


#[cfg(test)]
mod tests {
        use super::*;

    #[test]
    fn next() {
        let mut fl =Freelist::from([0, 1, 1, 2]);
        fl.remove(0);
        fl.remove(2);
        let mut iter = IntoIterFl::new(fl);

        assert_eq!(iter.next(), Some(1)); 
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None); 

        let mut iter = IntoIterFl::new(Freelist::from([0, 1, 2]));
        for i in [0, 1, 2] { 
            assert_eq!(iter.next(), Some(i));
        }
        assert_eq!(iter.next(), None);
    }

        #[test]
    fn next_back() {
        let mut fl =Freelist::from([0, 1, 1, 2]);
        fl.remove(0);
        fl.remove(2);
        let mut iter = IntoIterFl::new(fl);

        assert_eq!(iter.next_back(), Some(2)); 
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next_back(), None); 

        let mut iter = IntoIterFl::new(Freelist::from([0, 1, 2]));
        for i in [2, 1, 0] { 
            assert_eq!(iter.next_back(), Some(i));
        }
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn size_hint() {
        let mut fl =Freelist::from([0, 1, 1, 2]);
        fl.remove(0);
        fl.remove(2);
        let mut iter = IntoIterFl::new(fl);
        assert_eq!(iter.size_hint(), (4, Some(4)));
        iter.next();
        assert_eq!(iter.size_hint(), (2, Some(2)));
    }
}