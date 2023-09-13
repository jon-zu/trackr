# Trackr

Simple derive-based approach to track modifications of a 
structure in a flag set, which is provided by the `bitflags` create

## Example:

```rust
// Derive trackr::Tracked here
#[derive(Tracked, Default)]
pub struct Sample {
    // Flag field has to be marked, It's always `StructName`Flags
    #[track(flag)]
    tracker_flags: SampleFlags,
    // Private field, with tracker methods marked as public
    #[track(pub_)]
    a: u8,
    // Private field, with private tracker methods
    b: String,
    // Public field, with public tracker methods
    pub c: Vec<usize>,
    // Skip this field for tracking
    #[track(skip)]
    pub d: u32,
}
```

## Usage

Tracked fields are accesible via the `'field'_mut()`, which yields a `TrackedField` struct, which
holds a mutable reference to the field and the flag and offers mutable ways to update the value
and setting the changed flag in the background. The un-forced like `set` operations check If the value was changed,
thus requiring `PartialEq` implemented for the type. However there are also forced variants like `forced_set` which 
set the changed flag regard.