#![doc = include_str!("../doc/lib.md")]

use std::{hint::unreachable_unchecked, ops::{Index, IndexMut}, mem::replace};


#[derive(PartialEq, Debug, Clone)]
enum Slot<T> {
    Value(T),
    Next(usize),
    Empty
}

impl <T>Slot<T> {

    #[inline(always)]
    const fn from_value(value: T) -> Self {
        Self::Value(value)
    }

    #[inline(always)]
    unsafe fn to_some_unchecked(self) -> Option<T> {
        match self {
            Slot::Value(value) => Some(value),
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline(always)]
    unsafe fn as_value_unchecked(&self) -> &T {
        match self {
            Slot::Value(value) => value,
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline(always)]
    unsafe fn as_value_unchecked_mut(&mut self) -> &mut T {
        match self {
            Slot::Value(value) => value,
            _ => unsafe { unreachable_unchecked() }
        }
    }

}

#[doc = include_str!("../doc/freelist.md")]
#[derive(Debug, Clone)]
pub struct Freelist<T> {
    slots: Vec<Slot<T>>,
    next: Slot<T>,
    filled_length: usize,
}


impl<T> Freelist<T> {

    #[inline]
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
        let item = Slot::Value(value);
        match self.next {
            Slot::Next(index) => unsafe {
                self.next = replace(self.slots.get_unchecked_mut(index), item);
                index
            },
            _ => {
                self.slots.push(item);
                self.filled_length - 1
            }
        }
    }

    /// Returns the next available index OR total length of the list if full.
    #[inline]
    pub fn next_available(&self) -> usize {
        match self.next {
            Slot::Next(index) => index,
            _ => self.filled_length
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
    pub fn get(&self, index: usize) -> Option<&T> {
        match self.slots[index] {
            Slot::Value(ref value) => Some(value),
            _ => None,
        }
    }

    /// Returns a mutable reference to the element at the given index,
    /// or `None` if the index is a free slot.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self.slots[index] {
            Slot::Value(ref mut value) => Some(value),
            _ => None,
        }
    }

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
        self.slots.iter().filter_map(|c| match c {
            Slot::Value(value) => Some(value),
            _ => None
        })
    }

    /// Returns a mutable iterator over the freelist.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.iter_mut().filter_map(|c| match c {
            Slot::Value(value) => Some(value),
            _ => None
        })
    }

    /// Converts the freelist into an iterator, dropping any empty slots.
    pub fn into_iter(self)  -> impl Iterator<Item = T> {
        self.slots.into_iter().filter_map(|c| match c {
            Slot::Value(value) => Some(value),
            _ => None
        })
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
            slots: data.into_iter()
                .map(Slot::from_value)
                .collect(),
        }
    }
}

impl<T, const N: usize> From<[T; N]> for Freelist<T> {
    fn from(data: [T; N]) -> Self {
        Self {
            filled_length: N,
            next: Slot::Empty,
            slots: data.into_iter()
                .map(Slot::from_value)
                .collect(),
        }
    }
}

impl<T> FromIterator<T> for Freelist<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut len = 0;
        let data = iter.into_iter()
            .inspect(|_| len += 1)
            .map(Slot::from_value)
            .collect();
        
        Self {
            slots: data,
            filled_length: len,
            next: Slot::Empty
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{
        Slot::*,
        Freelist
    };
    

    #[test]
    fn push() {
        let mut list = Freelist::<f32>::new();

        list.push(0.0);
        list.push(1.0);
        list.push(2.0);

        assert_eq!(list.slots, vec![Value(0.0), Value(1.0), Value(2.0)]);

        println!("{:?}", list.slots);

    }

    #[test]
    fn remove() {
        let mut list = Freelist::<f32> {
            slots: vec![Value(0.0), Value(1.0), Value(2.0)],
            next: Empty,
            filled_length: 3,
        };

        let removed = list.remove(1);

        assert_eq!(removed, Some(1.0));
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

}