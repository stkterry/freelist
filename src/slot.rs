use std::hint::unreachable_unchecked;



#[derive(PartialEq, Debug, Clone)]

/// Container struct for items in Freelist
pub(super) enum Slot<T> {
    /// Contains a value `T`
    Value(T),
    /// Contains an index pointing to the next available slot
    Next(usize),
    /// An empty / terminal slot
    Empty
}

impl <T>Slot<T> {

    #[inline(always)]
    /// Converts [`Slot::Value(T)`](Slot::Value) into [`Some(T)`](Option::Some)
    /// 
    /// Calling on any other variant is UB.
    pub(super) unsafe fn to_some_unchecked(self) -> Option<T> {
        match self {
            Slot::Value(value) => Some(value),
            // Because this is only used internally, we can ensure we don't end up here.
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline(always)]
    /// Converts [`&Slot::Value(T)`](Slot::Value) into `&T`
    /// 
    /// Calling on any other variant is UB.
    pub(super) unsafe fn as_value_unchecked(&self) -> &T {
        match self {
            Slot::Value(value) => value,
            // Because this is only used internally, we can ensure we don't end up here.
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline(always)]
    /// Converts [`&mut Slot::Value(T)`](Slot::Value) into `&mut T`
    /// 
    /// Calling on any other variant is UB.
    pub(super) unsafe fn as_value_unchecked_mut(&mut self) -> &mut T {
        match self {
            Slot::Value(value) => value,
            // Because this is only used internally, we can ensure we don't end up here.
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline]
    pub(super) const fn is_value(&self) -> bool {
        if let Slot::Value(_) = self { true } else { false }
    }
}

impl <T> From<T> for Slot<T> {
    #[inline(always)]
    fn from(value: T) -> Self { Self::Value(value) }
}

impl <T> From<Slot<T>> for Option<T> {
    #[inline]
    fn from(value: Slot<T>) -> Self {
        match value {
            Slot::Value(value) => Some(value),
            _ => None
        }
    }
}

impl <'a, T> From<&'a Slot<T>> for Option<&'a T> {
    #[inline]
    fn from(value: &'a Slot<T>) -> Self {
        match value {
            Slot::Value(value) => Some(value),
            _ => None
        }
    }
}

impl <'a, T: 'a> From<&'a mut Slot<T>> for Option<&'a mut T> {
    #[inline]
    fn from(value: &'a mut Slot<T>) -> Self {
        match value {
            Slot::Value(value) => Some(value),
            _ => None
        }
    }
}




#[cfg(test)]
mod slot {

    use super::Slot;

    #[test]
    fn to_some_unchecked() {
        let slot = Slot::Value(5);
        assert_eq!(unsafe { slot.to_some_unchecked() }, Some(5));
    }

    #[test]
    fn as_value_unchecked() {
        let slot = Slot::Value(5);
        assert_eq!(unsafe { slot.as_value_unchecked() }, &5);
    }

    #[test]
    fn as_value_unchecked_mut() {
        let mut slot = Slot::Value(5);
        assert_eq!(unsafe { slot.as_value_unchecked_mut() }, &mut 5);
    }

    #[test]
    fn from_t() {
        let slot = Slot::from(5i32);
        assert_eq!(slot, Slot::Value(5));
    }

    #[test]
    fn from_slot() {
        let slot = Slot::Value(5);
        let none_slot = Slot::<i32>::Empty;
        assert_eq!(Option::from(slot), Some(5));
        assert_eq!(Option::<i32>::from(none_slot), None);
    }

}