
use std::{hint::unreachable_unchecked, ops::{Index, IndexMut}};

#[derive(PartialEq, Debug, Clone)]
enum Container<T> {
    Value(T),
    Next(usize),
    Empty
}

impl <T>Container<T> {
    
    #[inline(always)]
    fn is_value(&self) -> bool {
        match self {
            Container::Value(_) => true,
            _ => false
        }
    }

}


/// A contiguous, slotted, growable array type with in-place removable elements. 
#[derive(Debug, Clone)]
pub struct FreeList<T> {
    pub(crate) data: Vec<Container<T>>,
    pub(crate) free: Container<T>,
    len: usize,
}


impl<T> FreeList<T> {

    /// Appends an element to the first free slot or back of the list.
    /// 
    /// Returns the index of the element.
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
                self.len
            }
        }
    }

    #[inline]
    /// Returns the next available index OR total length of the list if full.
    pub fn next_available(&self) -> usize {
        match self.free {
            Container::Next(index) => index,
            _ => self.len
        }
    }

    #[inline]
    /// Removes and returns the value at the given index or None if the index is empty.
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


    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().filter_map(|c| match c {
            Container::Value(value) => Some(value),
            _ => None
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().filter_map(|c| match c {
            Container::Value(value) => Some(value),
            _ => None
        })
    }

    pub fn into_iter(self)  -> impl Iterator<Item = T> {
        self.data.into_iter().filter_map(|c| match c {
            Container::Value(value) => Some(value),
            _ => None
        })
    }

}

impl<T> Default for FreeList<T> {
    #[inline]
    fn default() -> Self {
        Self { data: vec![], free: Container::Empty, len: 0 }
    }
}

impl<T> Index<usize> for FreeList<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        use Container::*;
        match &self.data[index] {
            Value(element) => element,
            _ => { panic!("oh no!") }
        }
    }
}

impl<T> IndexMut<usize> for FreeList<T> {

    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        use Container::*;
        match &mut self.data[index] {
            Value(element) => element,
            _ => { panic!("oh no!") }
        }
    }
}

impl<T> From<Vec<T>> for FreeList<T> {
    fn from(data: Vec<T>) -> Self {
        Self {
            len: data.len(),
            data: data.into_iter()
                .map(|datum| Container::Value(datum))
                .collect(),
            free: Container::Empty,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{
        Container::*,
        FreeList
    };
    

    #[test]
    fn push() {
        let mut list = FreeList::<f32>::default();

        list.push(0.0);
        list.push(1.0);
        list.push(2.0);

        assert_eq!(list.data, vec![Value(0.0), Value(1.0), Value(2.0)]);

        println!("{:?}", list.data);

    }

    #[test]
    fn remove() {
        let mut list = FreeList::<f32> {
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
        let mut list = FreeList::<f32> {
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
        let mut list = FreeList::<f32> {
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
        let mut list = FreeList::<f32> {
            data: vec![Value(0.0), Value(1.0), Value(2.0)],
            free: Empty,
            len: 3,
        };

        list.clear();
        assert_eq!(list.free, Empty);
        assert_eq!(list.data, vec![]);
    }

}