use std::ops::{AddAssign, Deref, DerefMut};
use bitflags::Flags;

/// A tracked field in a tracked struct
/// which sets a flag when the value is changed.
#[derive(Debug)]
pub struct TrackedField<'s, T, F> {
    flag: F,
    track_flags: &'s mut F,
    value: &'s mut T,
}

impl<'s, T, F> TrackedField<'s, T, F>
where
    F: bitflags::Flags + Clone,
{
    /// Create a new tracked field
    pub fn new(flag: F, track_flags: &'s mut F, value: &'s mut T) -> Self {
        Self {
            flag,
            track_flags,
            value,
        }
    }

    /// Internal helper method to set the flag
    fn set_flag(&mut self) {
        self.track_flags.insert(self.flag.clone());
    }

    /// Forces an update, regardless of whether the value has changed.
    pub fn force_set(&mut self, value: T) {
        *self.value = value;
        self.set_flag();
    }

    /// Updates the value via a closure
    /// It should return true if the value has changed.
    pub fn update(&mut self, f: impl FnOnce(&mut T) -> bool) -> bool {
        if f(self.value) {
            self.set_flag();
            true
        } else {
            false
        }
    }

    /// Forces an update via a closure
    pub fn force_update(&mut self, f: impl FnOnce(&mut T)) {
        f(self.value);
        self.set_flag();
    }

    /// Update with an option closure, only sets the flag is Some is returned
    /// useful for checked updates
    /// Returns true if the value was updated
    pub fn update_opt(&mut self, f: impl FnOnce(&mut T) -> Option<T>) -> bool {
        if let Some(new) = f(self.value) {
            *self.value = new;
            self.set_flag();
            return true;
        }
        false
    }
}

impl<T, F> TrackedField<'_, T, F>
where
    T: PartialEq,
    F: bitflags::Flags + Clone,
{
    /// Set the value to the new one,
    /// but only set the flag whether the value changes
    pub fn set(&mut self, value: T) {
        if &value != self.value {
            self.force_set(value);
        }
    }
}

impl<T, F> Deref for TrackedField<'_, T, F> {
    type Target = T;

    /// Deref the Field value
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T, F> DerefMut for TrackedField<'_, T, F>
where
    F: bitflags::Flags + Clone,
{
    /// Derefs the field value into a mutable reference
    /// but also marks the value as changed
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.set_flag();
        self.value
    }
}

impl<T, F> AddAssign<T> for TrackedField<'_, T, F>
where
    T: AddAssign<T>,
    F: bitflags::Flags + Clone,
{
    fn add_assign(&mut self, rhs: T) {
        self.force_update(|v| v.add_assign(rhs));
    }
}



pub trait TrackedStruct {
    type Flags: bitflags::Flags;

    fn flags(&self) -> Self::Flags;
    fn flags_mut(&mut self) -> &mut Self::Flags;

    /// Returns None if no flags are set, otherwise resets the flags and returns them
    fn take_updates(&mut self) -> Option<Self::Flags> {
        let flags = std::mem::replace(self.flags_mut(), <Self::Flags as bitflags::Flags>::empty());
        if flags.is_empty() {
            None
        } else {
            Some(flags)
        }
    }
}


#[doc(hidden)]
pub mod __reexport {
    pub use bitflags::*;
}
pub use trackr_derive::*;
