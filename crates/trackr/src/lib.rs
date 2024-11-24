use std::ops::{Deref, DerefMut};

/// A tracked field in a tracked struct
/// which sets a flag when the value is changed.
#[derive(Debug)]
pub struct TrackedField<'s, T, F> {
    flag: F,
    value: &'s mut T,
    track_flags: &'s mut F,
}

impl<'s, T, F> TrackedField<'s, T, F>
where
    F: bitflags::Flags + Clone,
{
    /// Create a new tracked field
    pub fn new(flag: F, value: &'s mut T, track_flags: &'s mut F) -> Self {
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
    pub fn update(&mut self, f: impl FnOnce(&mut T) -> bool) {
        if f(self.value) {
            self.set_flag();
        }
    }

    /// Forces an update via a closure
    pub fn force_update(&mut self, f: impl FnOnce(&mut T)) {
        f(self.value);
        self.set_flag();
    }
}

impl<'s, T, F> TrackedField<'s, T, F>
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

impl<'s, T, F> Deref for TrackedField<'s, T, F> {
    type Target = T;

    /// Deref the Field value
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'s, T, F> DerefMut for TrackedField<'s, T, F>
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

pub use bitflags::*;
pub use trackr_derive::*;
