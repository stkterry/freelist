
mod iter;
mod iter_mut;
mod into_iter;

pub use into_iter::IntoIterFl;
pub use iter_mut::IterMutFl;
pub use iter::IterFl;

use crate::Slot;

#[inline(always)]
pub(super) const fn size_hint<T>(start: usize, end: usize) -> (usize, Option<usize>) {
    let len = (end - start) / std::mem::size_of::<Slot<T>>();
    (len, Some(len))
}