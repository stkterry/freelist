use std::{mem::{self, ManuallyDrop, MaybeUninit}, num::NonZeroUsize, ops::{Index, IndexMut}, ptr};

type Nzu = NonZeroUsize;

#[derive(Debug, Clone)]
struct Container<T> {
    datum: ManuallyDrop<T>,
    next: Option<Nzu>,
}

impl <T> Container<T> {

    #[inline]
    unsafe fn empty() -> Self {
        Self {
            datum: unsafe { MaybeUninit::uninit().assume_init() },
            next: None,
        }
    }

    #[inline]
    fn new(datum: T) -> Self {
        Self {
            datum: ManuallyDrop::new(datum),
            next: None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Freelist2<T> {
    data: Vec<Container<T>>,
    next: Option<Nzu>,
}

impl<T> Drop for Freelist2<T> {
    fn drop(&mut self) {
        let raw = self.data.as_mut_ptr();
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
                raw.add(1),
                self.data.len() - 1,
            ));
            self.data.set_len(0);
        }
    }
}

impl<T> Freelist2<T> {
    pub fn new() -> Self {
        Self {
            data: unsafe { vec![Container::empty()] },
            next: None,
        }
    }

    #[inline]
    pub fn reserve(&mut self, n: usize) {
        self.data.reserve_exact(n);
    }

    pub fn push(&mut self, datum: T) -> usize {
        match self.next {
            None => {
                self.data.push(Container::new(datum));
                self.data.len() - 2
            }
            Some(idx) => {
                let node = unsafe { self.data.get_unchecked_mut(idx.get()) };
                self.next = node.next.take();
                let mut prev = mem::replace(
                    &mut node.datum,
                    ManuallyDrop::new(datum)
                );
                unsafe { ManuallyDrop::drop(&mut prev) };
                idx.get() - 1
            }
        }
    }

    pub fn remove(&mut self, mut idx: usize) -> Option<T> {
        idx += 1;

        match self.data[idx].next {
            None => {
                self.data[idx].next = self.next;
                unsafe {
                    self.next = Some(Nzu::new_unchecked(idx));
                    Some(ManuallyDrop::take(&mut self.data[idx].datum))
                }
            }
            _ => None,
        }
    }

    pub unsafe fn remove_unchecked(&mut self, mut idx: usize) -> T {
        idx += 1;
        let node = unsafe { self.data.get_unchecked_mut(idx) };
        node.next = self.next;
        //self.data[idx].next = self.next;
        unsafe {
            self.next = Some(Nzu::new_unchecked(idx));
            ManuallyDrop::take(&mut node.datum)
        }
    }

    pub fn replace(&mut self, mut idx: usize, datum: T) -> Option<T> {
        idx += 1;

        if self.data[idx].next.is_none() && !self.next.is_some_and(|ndx| ndx.get() == idx) {
            let mut prev = mem::replace(
                &mut self.data[idx].datum,
                ManuallyDrop::new(datum)
            );
            return Some(unsafe { ManuallyDrop::take(&mut prev) })
        }
        None
    }

    /// Moves the value at the second index into the first, removing the second index.
    pub fn swap_remove(&mut self, mut idx: usize, mut ndx: usize) -> Option<T> {
        idx += 1;
        ndx += 1;
        if self.data[idx].next.is_none()
            && !self.next.is_some_and(|rdx| rdx.get() == idx)
            && self.data[ndx].next.is_none()
            && !self.next.is_some_and(|rdx| rdx.get() == ndx)
        {
           
            self.data[ndx].next = self.next;

            unsafe {

                self.next = Some(Nzu::new_unchecked(ndx));

                let i_datum: *mut ManuallyDrop<T> = &mut self.data[idx].datum;
                let n_datum: *mut ManuallyDrop<T> = &mut self.data[ndx].datum;
                ptr::swap(i_datum, n_datum);

                Some(ManuallyDrop::take(&mut self.data[ndx].datum))
            }

        } else {
            None
        }
    }

    /// Moves the value at the second index into the first, removing the second index.
    ///
    /// This is hilariously unsafe.
    pub unsafe fn swap_remove_unchecked(&mut self, mut idx: usize, mut ndx: usize) -> T {
        idx += 1;
        ndx += 1;

        unsafe {
            self.data.get_unchecked_mut(idx).next = self.next;
        }

        unsafe {

            self.next = Some(Nzu::new_unchecked(ndx));

            let i_datum: *mut ManuallyDrop<T> = &mut self.data.get_unchecked_mut(idx).datum;
            let n_datum: *mut ManuallyDrop<T> = &mut self.data.get_unchecked_mut(ndx).datum;
            ptr::swap(i_datum, n_datum);

            ManuallyDrop::take(&mut self.data.get_unchecked_mut(ndx).datum)
        }
    }

    pub unsafe fn replace_unchecked(&mut self, mut idx: usize, datum: T) -> T {
        idx += 1;

        let mut prev = mem::replace(
            &mut self.data[idx].datum,
            ManuallyDrop::new(datum)
        );
        unsafe { ManuallyDrop::take(&mut prev) }
    }

    pub fn delete(&mut self, mut idx: usize) {
        idx += 1;
        if self.data[idx].next.is_none() {
            self.data[idx].next = self.next;
            self.next = Some(unsafe { Nzu::new_unchecked(idx) });
        }
    }

    pub unsafe fn delete_unchecked(&mut self, mut idx: usize) {
        idx += 1;
        self.data[idx].next = self.next;
        self.next = Some(unsafe { Nzu::new_unchecked(idx) });
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, idx: usize) -> &T {
        unsafe { &self.data.get_unchecked(idx+1).datum }
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, idx: usize) -> &mut T {
        unsafe { &mut self.data.get_unchecked_mut(idx+1).datum }
    }


    pub fn to_vec(mut self) -> Vec<T> {
        let ndx = match self.next {
            Some(n) => n.get(),
            None => 0
        };

        self.data
            .iter_mut()
            .enumerate()
            .skip(1)
            .filter_map(|(idx, v)| {
                if idx == ndx { return None }
                match v.next {
                    None => unsafe { Some(ManuallyDrop::take(&mut v.datum)) }
                    _ => None,
                }
            })
            .collect()
    }

}
impl<T> From<Vec<T>> for Freelist2<T> {
    fn from(data: Vec<T>) -> Self {

        if data.len() == 0 { return Self::new() }

        let data = vec![unsafe { Container::empty() }]
            .into_iter()
            .chain(data.into_iter().map(|datum| Container::new(datum)))
            .collect();

        Self {
            data,
            next: None,
        }
    }
}

impl<T: Clone> Freelist2<T> {
    pub fn clone_as_vec(&self) -> Vec<T> {
        let ndx = match self.next {
            Some(n) => n.get(),
            None => 0
        };

        self.data
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(idx, v)| {
                if idx == ndx { return None }
                match v.next {
                    Some(_) => None,
                    None => {
                        let mut datum = v.datum.clone();
                        unsafe { Some(ManuallyDrop::take(&mut datum)) }
                    }
                }
            })
            .collect()
    }
}

impl<T> Into<Vec<T>> for Freelist2<T> {
    #[inline]
    fn into(self) -> Vec<T> { self.to_vec() }
}

impl<T> Index<usize> for Freelist2<T> {
    type Output = T;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        &self.data[idx+1].datum
    }
}

impl<T> IndexMut<usize> for Freelist2<T> {

    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.data[idx].datum

    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn from_and_to_vec() {
        let vec = vec![3, 14, 11, 42];
        let list = Freelist2::from(vec.clone());

        assert_eq!(vec, list.to_vec());
    }


    #[test]
    fn push() {
        let mut list = Freelist2::new();
        list.push(11);
        list.push(17);
        list.push(18);
        assert_eq!(
            vec![11, 17, 18],
            list.to_vec()
        );
    }

    #[test]
    fn remove() {
        let mut list = Freelist2::new();
        list.push(11);
        list.push(17);
        list.push(18);
        list.remove(1);
        list.push(15);
        list.remove(0);
        list.remove(0);

        assert_eq!(
            vec![15, 18],
            list.to_vec()
        );
    }



    #[test]
    fn remove_unchecked() {
        let mut list = Freelist2::new();
        list.push(11);
        list.push(17);
        list.push(18);
        unsafe { list.remove_unchecked(1) };
        list.push(15);
        unsafe { list.remove_unchecked(0) };

        assert_eq!(
            vec![15, 18],
            list.to_vec()
        );
    }

    #[test]
    fn replace() {
        let mut list = Freelist2::from(vec![10, 13, 12]);

        let replaced = list.replace(1, 11);

        assert_eq!(replaced, Some(13));
        assert_eq!(
            vec![10, 11, 12],
            list.to_vec()
        )
       
    }

    #[test]
    fn replace_unchecked() {
        let mut list = Freelist2::from(vec![10, 13, 12]);
        let replaced = unsafe { list.replace_unchecked(1, 11) };

        assert_eq!(replaced, 13);
        assert_eq!(
            vec![10, 11, 12],
            list.to_vec()
        )  
    }

    #[test]
    fn delete() {
        let mut list = Freelist2::from(vec![10, 13, 12]);

        let nothing = list.delete(2);
        list.delete(2);

        assert_eq!(nothing, ());
        assert_eq!(
            vec![10, 13],
            list.to_vec()
        )
    }

    #[test]
    fn delete_unchecked() {
        let mut list = Freelist2::from(vec![10, 13, 12]);

        let nothing = unsafe { list.delete_unchecked(2) };

        assert_eq!(nothing, ());
        assert_eq!(
            vec![10, 13],
            list.to_vec()
        )
    }

    #[test]
    fn swap_remove() {
        let mut list = Freelist2::from(vec![10, 13, 12, 42]);

        let removed = list.swap_remove(1, 2);
        assert_eq!(removed, Some(13));
        let removed = list.swap_remove(1, 2);
        assert_eq!(removed, None);

        assert_eq!(
            vec![10, 12, 42],
            list.to_vec()
        )
    }

    #[test]
    fn swap_remove_unchecked() {
        let mut list = Freelist2::from(vec![10, 13, 12, 42]);

        let removed = unsafe { list.swap_remove_unchecked(1, 2) };

        assert_eq!(removed, 13);
        assert_eq!(
            vec![10, 12, 42],
            list.to_vec()
        )
    }
}