
use std::{hint::unreachable_unchecked, ops::{Index, IndexMut}};


#[derive(PartialEq, Debug, Clone)]
enum Container<T> {
    Value(T),
    Next(usize),
    Empty
}

impl <T>Container<T> {
    
    #[inline(always)]
    const fn is_value(&self) -> bool {
        match self {
            Container::Value(_) => true,
            _ => false
        }
    }

    #[inline(always)]
    const fn from_value(value: T) -> Self {
        Self::Value(value)
    }

}

#[doc = include_str!("../doc/freelist.md")]
#[derive(Debug, Clone)]
pub struct Freelist<T> {
    pub(crate) data: Vec<Container<T>>,
    pub(crate) free: Container<T>,
    len: usize,
}


impl<T> Freelist<T> {

    #[inline]
    pub const fn new() -> Self { 
        Self { 
            data: Vec::new(),
            free: Container::Empty,
            len: 0
        }
    }

    /// Appends an element to the first free slot or back of the list
    /// and returns the index of insertion.
    #[inline]
    pub fn push(&mut self, value: T) -> usize {
        self.len += 1;
        let item = Container::Value(value);
        match self.free {
            Container::Next(index) => {
                self.free = std::mem::replace(&mut self.data[index], item);
                index
            },
            _ => {
                self.data.push(item);
                self.len - 1
            }
        }
    }

    /// Returns the next available index OR total length of the list if full.
    #[inline]
    pub fn next_available(&self) -> usize {
        match self.free {
            Container::Next(index) => index,
            _ => self.len
        }
    }

    /// Removes and returns the value at the given index or None if the index is empty.
    /// 
    /// This operation preserves ordering and is always *O*(1).
    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {
 
        if !self.data[index].is_value() { return None }

        self.len -= 1;

        match std::mem::replace(
            &mut self.data[index],
            std::mem::replace(&mut self.free, Container::Next(index))
        ) {
            Container::Value(value) => Some(value),
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline]
    /// Returns the number of filled slots in the list.
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    /// Returns the length of the list, including empty slots.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    #[inline]
    /// Returns the number of free slots in the list.
    pub fn free(&self) -> usize {
        self.data.len() - self.len
    }

    #[inline]
    /// Clears the freelist, removing all values.
    pub fn clear(&mut self) {
        self.data.clear();
        self.free = Container::Empty;
        self.len = 0;
    }

    #[inline]
    /// Reserves the minimum capacity for at least `n` more elements.  This function will
    /// take into account any free slots within the underlying list.
    pub fn reserve(&mut self, n: usize) {
        self.data.reserve_exact(n - self.free());
    }


    /// Returns a reference to the element at the given index,
    /// or `None` if the index is a free slot.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        match self.data[index] {
            Container::Value(ref value) => Some(value),
            _ => None,
        }
    }

    /// Returns a mutable reference to the element at the given index,
    /// or `None` if the index is a free slot.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self.data[index] {
            Container::Value(ref mut value) => Some(value),
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
        unsafe { match self.data.get_unchecked(index) {
            Container::Value(value) => value,
            _ => unreachable_unchecked()
        }}
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
        unsafe { match self.data.get_unchecked_mut(index) {
            Container::Value(value) => value,
            _ => unreachable_unchecked()
        }}
    }


    /// Returns an iterator over the freelist.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().filter_map(|c| match c {
            Container::Value(value) => Some(value),
            _ => None
        })
    }

    /// Returns a mutable iterator over the freelist.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().filter_map(|c| match c {
            Container::Value(value) => Some(value),
            _ => None
        })
    }

    /// Converts the freelist into an iterator, dropping any empty slots.
    pub fn into_iter(self)  -> impl Iterator<Item = T> {
        self.data.into_iter().filter_map(|c| match c {
            Container::Value(value) => Some(value),
            _ => None
        })
    }

}

impl<T> Default for Freelist<T> {
    fn default() -> Self {
        Self { data: Vec::new(), free: Container::Empty, len: 0 }
    }
}

impl<T> Index<usize> for Freelist<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        use Container::*;
        match &self.data[index] {
            Value(element) => element,
            _ => panic!("Attempted to access an empty slot. Index: {index}")
        }
    }
}

impl<T> IndexMut<usize> for Freelist<T> {

    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        use Container::*;
        match &mut self.data[index] {
            Value(element) => element,
            _ => panic!("Attempted to access an empty slot. Index: {index}")
        }
    }
}

impl<T> From<Vec<T>> for Freelist<T> {
    fn from(data: Vec<T>) -> Self {
        Self {
            len: data.len(),
            free: Container::Empty,
            data: data.into_iter()
                .map(Container::from_value)
                .collect(),
        }
    }
}

impl<T, const N: usize> From<[T; N]> for Freelist<T> {
    fn from(data: [T; N]) -> Self {
        Self {
            len: N,
            free: Container::Empty,
            data: data.into_iter()
                .map(Container::from_value)
                .collect(),
        }
    }
}

impl<T> FromIterator<T> for Freelist<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut len = 0;
        let data = iter.into_iter()
            .inspect(|_| len += 1)
            .map(Container::from_value)
            .collect();
        
        Self {
            data,
            len,
            free: Container::Empty
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{
        Container::*,
        Freelist
    };
    

    #[test]
    fn push() {
        let mut list = Freelist::<f32>::new();

        list.push(0.0);
        list.push(1.0);
        list.push(2.0);

        assert_eq!(list.data, vec![Value(0.0), Value(1.0), Value(2.0)]);

        println!("{:?}", list.data);

    }

    #[test]
    fn remove() {
        let mut list = Freelist::<f32> {
            data: vec![Value(0.0), Value(1.0), Value(2.0)],
            free: Empty,
            len: 3,
        };

        let removed = list.remove(1);

        assert_eq!(removed, Some(1.0));
        assert_eq!(list.free, Next(1));
        assert_eq!(list.data, vec![Value(0.0), Empty, Value(2.0)]);
    }

    #[test]
    fn remove_then_push() {
        let mut list = Freelist::<f32> {
            data: vec![Value(0.0), Value(1.0), Value(2.0)],
            free: Empty,
            len: 3,
        };

        list.remove(1);
        list.push(3.0);

        assert_eq!(list.free, Empty);
        assert_eq!(list.data, vec![Value(0.0), Value(3.0), Value(2.0)]);
    }

    #[test]
    fn remove_then_push_multiple() {
        let mut list = Freelist::<f32> {
            data: vec![Value(0.0), Value(1.0), Value(2.0)],
            free: Empty,
            len: 3,
        };

        list.remove(1);
        list.remove(2);
        list.push(3.0);
        list.push(4.0);
        list.push(5.0);

        assert_eq!(list.free, Empty);
        assert_eq!(list.data, vec![Value(0.0), Value(4.0), Value(3.0), Value(5.0)]);
    }

    #[test]
    fn clear() {
        let mut list = Freelist::<f32> {
            data: vec![Value(0.0), Value(1.0), Value(2.0)],
            free: Empty,
            len: 3,
        };

        list.clear();
        assert_eq!(list.free, Empty);
        assert_eq!(list.data, vec![]);
    }

}