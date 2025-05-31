use std::ptr;

use crate::{Freelist, Slot};



pub struct IntoIterFl<T> {
    start: *const Slot<T>,
    end: *const Slot<T>,
    _fl: Freelist<T>
}

impl<T> IntoIterFl<T> {
    #[inline]
    pub(super) fn new(freelist: Freelist<T>) -> Self {
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
            unsafe { match ptr::read(self.start) {
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

impl<T> DoubleEndedIterator for IntoIterFl<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if self.end == self.start { return None }
            unsafe { match ptr::read(self.end) {
                Slot::Value(value) => {
                    self.end = self.end.offset(-1);
                    return Some(value) 
                },
                _ => { self.end = self.end.offset(-1) }
            }}
        }
    }
}

impl<T> Drop for IntoIterFl<T> {
    fn drop(&mut self) { for _ in &mut * self { } }
}
