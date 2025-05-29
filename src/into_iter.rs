use std::{iter::FilterMap, vec::IntoIter};

use crate::{Freelist, Slot};


pub struct FreelistIter<T> {
    filter_map: FilterMap<IntoIter<Slot<T>>, fn(Slot<T>) -> Option<T>>,
}

impl <T> FreelistIter<T> {
    #[inline]
    pub(super) fn new(freelist: Freelist<T>) -> Self {
        Self {
            filter_map: freelist.slots
                .into_iter()
                .filter_map(Into::into)
        }
    }
}

impl <T> Iterator for FreelistIter<T> {
    type Item = T;
    
    #[inline]
    fn next(&mut self) -> Option<Self::Item> { self.filter_map.next() }

    #[inline]
    fn  size_hint(&self) -> (usize, Option<usize>) { self.filter_map.size_hint() }
}

impl <T> DoubleEndedIterator for FreelistIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> { self.filter_map.next_back() }
}