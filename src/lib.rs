
pub mod freelist2;

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

#[derive(Debug, Clone)]
pub struct FreeList<T> {
    pub(crate) data: Vec<Container<T>>,
    pub(crate) free: Container<T>,
}

impl<T> FreeList<T> {

    #[inline]
    pub fn push(&mut self, value: T) -> usize {
        
        let item = Container::Value(value);
        match self.free {
            Container::Next(index) => {
                self.free = std::mem::replace(&mut self.data[index], item);
                index
            },
            _ => {
                self.data.push(item);
                self.data.len() - 1
            }
        }
    }

    #[inline]
    pub fn next_available(&self) -> usize {
        match self.free {
            Container::Next(index) => index,
            _ => self.data.len()
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {
 
        if !self.data[index].is_value() { return None }

        match std::mem::replace(
            &mut self.data[index],
            std::mem::replace(&mut self.free, Container::Next(index))
        ) {
            Container::Value(value) => Some(value),
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
        self.free = Container::Empty;
    }

    #[inline]
    pub fn reserve(&mut self, n: usize) {
        self.data.reserve_exact(n);
    }

}

impl<T> Default for FreeList<T> {
    #[inline]
    fn default() -> Self {
        Self { data: vec![], free: Container::Empty }
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
            free: Empty
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
            free: Empty
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
            free: Empty
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
            free: Empty
        };

        list.clear();
        assert_eq!(list.free, Empty);
        assert_eq!(list.data, vec![]);
    }

}