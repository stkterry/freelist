#![doc = include_str!("../doc/lib.md")]

mod into_iter;
mod slot;

use std::{hint::unreachable_unchecked, ops::{Index, IndexMut}, mem::replace};
use into_iter::FreelistIter;
use slot::Slot;



#[doc = include_str!("../doc/freelist.md")]
#[derive(Debug, Clone)]
pub struct Freelist<T> {
    slots: Vec<Slot<T>>,
    next: Slot<T>,
    filled_length: usize,
}


impl<T> Freelist<T> {

    #[inline]
    /// Creates an empty Freelist
    pub const fn new() -> Self { 
        Self { 
            slots: Vec::new(),
            next: Slot::Empty,
            filled_length: 0
        }
    }

    /// Appends an element to the first free slot or back of the list
    /// and returns the index of insertion.
    #[inline]
    pub fn push(&mut self, value: T) -> usize {
        self.filled_length += 1;
        let src = Slot::Value(value);
        match self.next {
            Slot::Next(index) => unsafe {
                self.next = replace(self.slots.get_unchecked_mut(index), src);
                index
            },
            Slot::Empty => {
                self.slots.push(src);
                self.filled_length - 1
            },
            _ => unsafe { unreachable_unchecked() }
        }
    }

    /// Returns the next available index OR total length of the list if full.
    #[inline]
    pub fn next_available(&self) -> usize {
        match self.next {
            Slot::Next(index) => index,
            Slot::Empty => self.filled_length,
            _ => unsafe { unreachable_unchecked() }
        }
    }

    /// Removes and returns the value at the given index or None if the index is empty.
    /// 
    /// This operation preserves ordering and is always *O*(1).
    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {

        // The data struture guarantees the following operations are valid.
        // Next(index) -> self.next -> Value(value) -> return Some(value)
        match &mut self.slots[index] {
            value @ Slot::Value(_) => unsafe {
                self.filled_length -= 1;
                replace(value, replace(&mut self.next, Slot::Next(index)))
                    .to_some_unchecked()
            },
            _ => None
        }
    }

    #[inline]
    /// Returns the number of filled slots in the list.
    pub const fn filled(&self) -> usize {
        self.filled_length
    }

    #[inline]
    /// Returns the length of the list, including empty slots.
    pub fn size(&self) -> usize {
        self.slots.len()
    }

    #[inline]
    /// Returns the number of free slots in the list.
    pub fn free(&self) -> usize {
        self.slots.len() - self.filled_length
    }

    #[inline]
    /// Clears the freelist, removing all values.
    pub fn clear(&mut self) {
        self.slots.clear();
        self.next = Slot::Empty;
        self.filled_length = 0;
    }

    #[inline]
    /// Reserves the minimum capacity for at least `n` more elements.  This function will
    /// take into account any free slots within the underlying list.
    pub fn reserve(&mut self, n: usize) {
        self.slots.reserve_exact(n - self.free());
    }


    /// Returns a reference to the element at the given index,
    /// or `None` if the index is a free slot.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> { (&self.slots[index]).into() }

    /// Returns a mutable reference to the element at the given index,
    /// or `None` if the index is a free slot.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> { (&mut self.slots[index]).into() }

    /// Returns a reference to the element at the given index, without
    /// doing bounds checking or asserting the status of the slot.
    /// 
    /// See [`get`](Freelist::get) for a safe alternative.
    /// 
    /// # Safety
    /// Calling this method with an out-of-bounds index OR on an index that
    /// is already empty is [undefined behavior]: <https://doc.rust-lang.org/reference/behavior-considered-undefined.html>
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        unsafe { self.slots.get_unchecked(index).as_value_unchecked() }
    }

    /// Returns a mutable reference to the element at the given index, without
    /// doing bounds checking or asserting the slot contains a value.
    /// 
    /// See [`get_mut`](Freelist::get_mut) for a safe alternative.
    /// 
    /// # Safety
    /// Calling this method with an out-of-bounds index OR on an index that
    /// is already empty is [undefined behavior]: <https://doc.rust-lang.org/reference/behavior-considered-undefined.html>
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        unsafe { self.slots.get_unchecked_mut(index).as_value_unchecked_mut() }

    }


    /// Returns an iterator over the freelist.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().filter_map(Option::from)
    }

    /// Returns a mutable iterator over the freelist.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.iter_mut().filter_map(Option::from)
    }

}

impl<T> Default for Freelist<T> {
    fn default() -> Self {
        Self { slots: Vec::new(), next: Slot::Empty, filled_length: 0 }
    }
}

impl<T> Index<usize> for Freelist<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match &self.slots[index] {
            Slot::Value(element) => element,
            _ => panic!("attempted to access an empty slot")
        }
    }
}

impl<T> IndexMut<usize> for Freelist<T> {

    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match &mut self.slots[index] {
            Slot::Value(element) => element,
            _ => panic!("attempted to access an empty slot")
        }
    }
}

impl<T> From<Vec<T>> for Freelist<T> {
    fn from(data: Vec<T>) -> Self {
        Self {
            filled_length: data.len(),
            next: Slot::Empty,
            slots: data.into_iter().map(T::into).collect(),
        }
    }
}

impl<T, const N: usize> From<[T; N]> for Freelist<T> {
    fn from(data: [T; N]) -> Self {
        Self {
            filled_length: N,
            next: Slot::Empty,
            slots: data.into_iter().map(T::into).collect(),
        }
    }
}

impl<T> IntoIterator for Freelist<T> {
    type Item = T;
    type IntoIter = FreelistIter<T>;
    
    fn into_iter(self) -> Self::IntoIter { Self::IntoIter::new(self) }
}

impl<T> FromIterator<T> for Freelist<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut filled_length = 0;
        let data = iter.into_iter()
            .inspect(|_| filled_length += 1)
            .map(T::into)
            .collect();
        
        Self {
            slots: data,
            filled_length,
            next: Slot::Empty
        }
    }
}




#[cfg(test)]
mod freelist {
    use super::{
        Slot::*,
        Freelist
    };

    #[test]
    fn new() {
        let list = Freelist::<i32>::new();
        assert_eq!(list.slots, Vec::new());
        assert_eq!(list.filled_length, 0);
        assert_eq!(list.next, Empty);
    }


    #[test]
    fn push() {
        let mut list = Freelist::<f32>::new();

        list.push(0.0);
        let idx = list.push(1.0);
        list.push(2.0);

        assert_eq!(idx, 1);
        assert_eq!(list.slots, vec![Value(0.0), Value(1.0), Value(2.0)]);
        
    }

    #[test]
    fn next_available() {
        let mut list = Freelist::from([0, 1]);
        assert_eq!(list.next_available(), 2);
        list.remove(0);
        assert_eq!(list.next_available(), 0);
    }

    #[test]
    fn remove() {
        let mut list = Freelist::<f32> {
            slots: vec![Value(0.0), Value(1.0), Value(2.0)],
            next: Empty,
            filled_length: 3,
        };

        let removed = list.remove(1);
        let none_removed = list.remove(1);

        assert_eq!(removed, Some(1.0));
        assert_eq!(none_removed, None);
        assert_eq!(list.next, Next(1));
        assert_eq!(list.slots, vec![Value(0.0), Empty, Value(2.0)]);
    }

    #[test]
    fn remove_then_push() {
        let mut list = Freelist::<f32> {
            slots: vec![Value(0.0), Value(1.0), Value(2.0)],
            next: Empty,
            filled_length: 3,
        };

        list.remove(1);
        list.push(3.0);

        assert_eq!(list.next, Empty);
        assert_eq!(list.slots, vec![Value(0.0), Value(3.0), Value(2.0)]);
    }

    #[test]
    fn remove_then_push_multiple() {
        let mut list = Freelist::<f32> {
            slots: vec![Value(0.0), Value(1.0), Value(2.0)],
            next: Empty,
            filled_length: 3,
        };

        list.remove(1);
        list.remove(2);
        list.push(3.0);
        list.push(4.0);
        list.push(5.0);

        assert_eq!(list.next, Empty);
        assert_eq!(list.slots, vec![Value(0.0), Value(4.0), Value(3.0), Value(5.0)]);
    }

    #[test]
    fn clear() {
        let mut list = Freelist::<f32> {
            slots: vec![Value(0.0), Value(1.0), Value(2.0)],
            next: Empty,
            filled_length: 3,
        };

        list.clear();
        assert_eq!(list.next, Empty);
        assert_eq!(list.slots, vec![]);
    }

    #[test]
    fn reserve() {
        let mut list = Freelist::<i32>::new();
        list.reserve(16);
        assert_eq!(list.slots.capacity(), 16);
    }

    #[test]
    fn filled() {
        let mut list = Freelist::from([0, 1, 2, 3]);
        assert_eq!(list.filled(), 4);
        list.remove(2);
        assert_eq!(list.filled(), 3);
    }

    #[test]
    fn size() {
        let mut list = Freelist::from([0, 1, 2, 3]);
        assert_eq!(list.size(), 4);
        list.remove(2);
        assert_eq!(list.size(), 4);
    }

    #[test]
    fn free() {
        let mut list = Freelist::from([0, 1, 2, 3]);
        assert_eq!(list.free(), 0);
        list.remove(2);
        assert_eq!(list.free(), 1);
    }

    #[test]
    fn get() {
        let mut list = Freelist::from([0, 1, 2, 3]);
        assert_eq!(list.get(2), Some(&2));
        list.remove(2);
        assert_eq!(list.get(2), None);
    }

    #[test]
    fn get_mut() {
        let mut list = Freelist::from([0, 1, 2, 3]);
        assert_eq!(list.get_mut(2), Some(&mut 2));
        list.remove(2);
        assert_eq!(list.get_mut(2), None);
    }


    #[test]
    fn get_unchecked() {
        let list = Freelist::from([0, 1, 2, 3]);
        assert_eq!(unsafe { list.get_unchecked(2) }, &2);
    }

    #[test]
    fn get_unchecked_mut() {
        let mut list = Freelist::from([0, 1, 2, 3]);
        assert_eq!(unsafe { list.get_unchecked_mut(2) }, &mut 2);
    }

    #[test]
    fn iter() {
        let arr = [0, 1, 2, 3];
        let mut list = Freelist::from(arr.clone());
        list.remove(1);
        let collected = list.iter().copied().collect::<Vec<i32>>();
        assert_eq!([0, 2, 3].as_slice(), collected);
    }

    #[test]
    fn iter_mut() {
        let arr = [0, 1, 2, 3];
        let mut list = Freelist::from(arr.clone());
        list.remove(1);
        let collected = list.iter_mut().map(|v| *v).collect::<Vec<i32>>();
        assert_eq!([0, 2, 3].as_slice(), collected);
    }


    #[test]
    fn default() {
        let list = Freelist::<i32>::default();
        let list2 = Freelist::<i32>::new();

        assert_eq!(list.slots, list2.slots);
        assert_eq!(list.next, list2.next);
        assert_eq!(list.filled_length, list2.filled_length);
    }

    #[test]
    fn index() {
        let list = Freelist::from([0, 1, 2]);
        assert_eq!(&list[1], &1);
    }

    #[test]
    #[should_panic]
    fn index_panic() {
        let mut list = Freelist::from([0, 1, 2]);
        list.remove(1);
        let _ = &list[1];
    }

    #[test]
    fn index_mut() {
        let mut list = Freelist::from([0, 1, 2]);
        assert_eq!(&mut list[1], &mut 1);
    }

    #[test]
    #[should_panic]
    fn index_mut_panic() {
        let mut list = Freelist::from([0, 1, 2]);
        list.remove(1);
        let _ = &mut list[1];
    }

    #[test]
    fn from_vec() {
        let list = Freelist::from(vec![0, 1, 2]);
        assert_eq!(list.slots, [Value(0), Value(1), Value(2)]);
    }

    #[test]
    fn from_arr() {
        let list = Freelist::from([0, 1, 2]);
        assert_eq!(list.slots, [Value(0), Value(1), Value(2)]);
    }

    #[test]
    fn into_iter() {
        let list = Freelist::from([0, 1, 2]);
        assert_eq!(list.into_iter().collect::<Vec<i32>>(), [0, 1, 2]);
    }

    #[test]
    fn from_iter() {
        let iter = [0, 1, 2].into_iter();
        let list = Freelist::from_iter(iter);

        assert_eq!(list.slots, [Value(0), Value(1), Value(2)]);
    }

    #[test]
    fn double_ended_iter() {
        let list = Freelist::from([0, 1, 2]);
        assert_eq!(list.into_iter().next_back(), Some(2))       
    }

}