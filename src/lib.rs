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

    /// Construct a new, empty `Freelist<T>`
    /// 
    /// The list will not allocate until elements are pushed onto it.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # #![allow(unused_mut)]
    /// use fffl::Freelist;
    /// 
    /// let mut fl: Freelist<i32> = Freelist::new();
    /// ```
    #[inline]
    pub const fn new() -> Self { 
        Self { 
            slots: Vec::new(),
            next: Slot::Empty,
            filled_length: 0
        }
    }

    /// Constructs a new, empty `Freelist<T>` with at least the specified capacity.
    /// 
    /// This function shares all of the same details as its underlying Vec: [`Vec::with_capacity`]
    /// 
    /// # Example
    /// 
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl: Freelist<i32> = Freelist::with_capacity(10);
    /// 
    /// // The freelist contains no items, just capacity.
    /// assert_eq!(fl.size(), 0);
    /// assert!(fl.capacity() >= 10);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            next: Slot::Empty,
            filled_length: 0
        }
    }

    /// Appends an element to the first free slot (or back of the list)
    /// and returns the index of insertion.
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds `isize::MAX` *bytes*.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl = Freelist::from([1, 2]);
    /// let _ = fl.push(3);
    /// assert_eq!(fl.to_vec(), vec![1, 2, 3]);
    /// ```
    /// # Time complexity
    /// 
    /// Takes amortized *O*(1) time.  The freelist will first try to push into 
    /// previously freed slots before reallocating, if available.  *O*(*capacity*) time is taken to copy the
    /// freelist's elements to a larger allocation. This expensive operation is
    /// offset by the *capacity* *O*(1) insertions it allows.
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

    /// Returns the next available index.
    /// 
    /// If there are no free slots, this will be the length of the freelist and 
    /// therefore NOT a suitable slot to index.
    #[inline]
    pub fn next_available(&self) -> usize {
        match self.next {
            Slot::Next(index) => index,
            Slot::Empty => self.filled_length,
            _ => unsafe { unreachable_unchecked() }
        }
    }

    /// Removes and returns the value at position `index` within the freelist, or [`None`] if
    /// the slot was previously freed.
    /// 
    /// This operation preserves ordering and is always *O*(1).
    /// 
    /// # Panics
    /// 
    /// Panics if `index` is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl = Freelist::from(['a', 'b', 'c']);
    /// 
    /// assert_eq!(fl.remove(1), Some('b'));
    /// 
    /// assert_eq!(fl.to_vec(), vec!['a', 'c']);
    /// ```
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

    /// Returns the number of filled slots in the list.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl = Freelist::from([0, 1, 2]);
    /// 
    /// assert_eq!(fl.filled(), 3);
    /// let _ = fl.remove(1);
    /// assert_eq!(fl.filled(), 2);
    /// ```
    #[inline]
    pub const fn filled(&self) -> usize {
        self.filled_length
    }

    /// Returns the length of the list, including freed slots.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl = Freelist::from([0, 1, 2]);
    /// 
    /// assert_eq!(fl.size(), 3);
    /// let _ = fl.remove(1);
    /// assert_eq!(fl.size(), 3);
    /// ```
    #[inline]
    pub fn size(&self) -> usize {
        self.slots.len()
    }

    /// Returns the number of free slots in the list.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl = Freelist::from([0, 1, 2]);
    /// 
    /// assert_eq!(fl.free(), 0);
    /// let _ = fl.remove(1);
    /// assert_eq!(fl.free(), 1);
    /// ```
    #[inline]
    pub fn free(&self) -> usize {
        self.slots.len() - self.filled_length
    }

    /// Returns the total number of slots the freelist can hold without reallocating.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl: Freelist<i32> = Freelist::with_capacity(10);
    /// fl.push(42);
    /// assert!(fl.capacity() >= 10);
    /// 
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize { self.slots.capacity() }

    #[inline]
    /// Clears the freelist, removing all values.
    pub fn clear(&mut self) {
        self.slots.clear();
        self.next = Slot::Empty;
        self.filled_length = 0;
    }

    /// Converts the freelist into a `Vec<T>`, skipping free slots.
    pub fn to_vec(self) -> Vec<T> { 
        self.into_iter().collect() 
    }

    /// Reserves the minimum capacity for at least `additional` more elements to
    /// be inserted in the given `Freelist<T>`.  The function will account for
    /// previously freed slots.
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl = Freelist::from([1]);
    /// fl.reserve(10);
    /// assert!(fl.capacity() >= 11);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.slots.reserve_exact(additional - self.free());
    }

    /// Swaps all values to the front of the freelist.
    /// 
    /// After repeated calls to [`push`] and [`remove`], caching and index 
    /// peformance may worsen as a result of increasingly randomized 
    /// reads/writes to the freelist. This functions addresses this by 
    /// shuffling values from the back of the freelist into free slots at 
    /// the front.
    /// 
    /// The function under the hood works thus:
    /// ```text
    /// Legend:
    ///     V - Value
    ///     F - Free / Empty Slot
    ///
    /// Before: freelist[V, F, F, V, V, F, V]
    /// After:  freelist[V, V, V, V, F, F, F]
    /// ```
    /// # Important Note
    /// 
    /// This function does **NOT** retain the original order and `indices` 
    /// returned by previous calls to [`push`] may no longer refer to their 
    /// values, therefore usefulness is somewhat limited.  The 
    /// upside is repeated calls to [`push`] may now have better 
    /// caching and index performance.
    /// 
    /// This is likely only beneficial for particularly sparse, large `Freelists`.
    /// 
    /// # Time Complexity
    /// 
    /// In general this function is guaranteed to have *O*(n) performance,
    /// where `n` is the size of the freelist. See [`size`]
    pub fn compactify(&mut self) {

        let mut iter = self.slots.iter_mut();
        self.next = Slot::Empty;

        loop {
            let hole = loop {
                if let Some(front) = iter.next() {
                    match front {
                        Slot::Value(_) => {},
                        r @ _ => break r
                    }
                } else { 
                    self.slots.truncate(self.filled_length);
                    return
                }
            };
            let plug = loop {
                if let Some(back) = iter.next_back() {
                    match back {
                        v @ Slot::Value(_) => break v,
                        _ => {}
                    }
                } else {
                    self.slots.truncate(self.filled_length); 
                    return 
                }
            };

            // In benchmarking this is a few percent faster than using
            // std::mem::swap(hole, plug).  It's safe because the function will 
            // truncate the duplicated values at any given 'plug' anyway. Thus 
            // we avoid violating memory safety.
            unsafe {
                std::ptr::copy_nonoverlapping(plug, hole, 1)
            }
        }
    }


    /// Returns a reference to the element at the given index,
    /// or `None` if the index is a free slot.
    /// 
    /// # Panics
    /// 
    /// Panics if `index` is out of bounds
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> { (&self.slots[index]).into() }

    /// Returns a mutable reference to the element at the given index,
    /// or `None` if the index is a free slot.
    /// 
    /// # Panics
    /// 
    /// Panics if `index` is out of bounds
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> { (&mut self.slots[index]).into() }

    /// Returns a reference to the element at the given index, without
    /// doing bounds checking or asserting the status of the slot.
    /// 
    /// See [`get`](Freelist::get) for a safe alternative.
    /// 
    /// # Safety
    /// 
    /// Calling this method with an out-of-bounds index OR on an index that
    /// is already empty is [undefined behavior](<https://doc.rust-lang.org/reference/behavior-considered-undefined.html>).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl = Freelist::from([1, 2, 4]);
    /// 
    /// unsafe {
    ///     assert_eq!(fl.get_unchecked(1), &2);
    /// }
    /// ```
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
    /// is already empty is [undefined behavior](<https://doc.rust-lang.org/reference/behavior-considered-undefined.html>).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl = Freelist::from([1, 2, 4]);
    /// 
    /// unsafe {
    ///     let val = fl.get_unchecked_mut(1);
    ///     *val = 13;
    /// }
    /// assert_eq!(fl.to_vec(), [1, 13, 4]);
    /// ```
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        unsafe { self.slots.get_unchecked_mut(index).as_value_unchecked_mut() }
    }

    /// Returns an iterator over the full freelist.
    /// 
    /// The iterator will skip over freed slots, returning only valid entries from start to end.
    /// 
    /// # Examples
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl = Freelist::from([1, 2, 4]);
    /// let mut iterator = fl.iter();
    /// 
    /// assert_eq!(iterator.next(), Some(&1));
    /// assert_eq!(iterator.next(), Some(&2));
    /// assert_eq!(iterator.next(), Some(&4));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().filter_map(Option::from)
    }

    /// Returns an iterator over the full freelist that allows modifying each value.
    /// 
    /// The iterator will skip over freed slots, returning only valid entries from start to end.
    /// 
    /// # Examples
    /// ```
    /// use fffl::Freelist;
    /// 
    /// let mut fl = Freelist::from([1, 2, 4]);
    /// for val in fl.iter_mut() {
    ///     *val += 2;
    /// }
    /// 
    /// assert_eq!(fl.to_vec(), [3, 4, 6]);
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.iter_mut().filter_map(Option::from)
    }

}

impl<T> Default for Freelist<T> {
    /// Creates an empty `Freelist<T>`.
    /// 
    /// The freelist will not allocate until elements are pushed into it.
    fn default() -> Self {
        Self { slots: Vec::new(), next: Slot::Empty, filled_length: 0 }
    }
}

impl<T> Index<usize> for Freelist<T> {
    type Output = T;

    /// Performs the indexing `(container[index])` operation. [Read more](<https://doc.rust-lang.org/std/ops/trait.Index.html#tymethod.index>)
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match &self.slots[index] {
            Slot::Value(element) => element,
            _ => panic!("attempted to access an empty slot")
        }
    }
}

impl<T> IndexMut<usize> for Freelist<T> {

    // Performs the mutable indexing `(container[index])` operation. [Read more](<https://doc.rust-lang.org/1.85.1/core/ops/trait.IndexMut.html#tymethod.index_mut>)
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

    #[test]
    fn with_capacity() {
        let list = Freelist::<i32>::with_capacity(10);
        assert_eq!(list.capacity(), 10);
    }

    #[test]
    fn to_vec() {
        let mut list = Freelist::from([1, 2, 3]);
        list.remove(1);
        assert_eq!(list.to_vec(), [1, 3]);
    }

    #[test]
    fn compactify() {
        let mut list = Freelist::from([1, 2, 3, 4, 5, 6, 7]);
        list.remove(1);
        list.remove(3);
        list.remove(5);

        list.compactify();

        assert_eq!(list.free(), 0);
        assert_eq!(list.size(), 4);
        assert_eq!(list.to_vec(), [1, 7, 3, 5]);
    }
}