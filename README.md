# Trackr

Simple derive-based approach to track modifications in a bitflag set, so each time a field is modified the corresponding flag is set.

## Notes

The crate is limited to a maximum of 128 fields per struct for now.

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

Tracked fields are accesible via the `'field'_mut()` returning a `TrackedField<T>` struct, holding a mutable reference to the field and offering multiple functions to update the field. The un-forced like `set` operations checks If the value was changed,
but requiring `PartialEq` for the type. There are also forced variants like `forced_set` which 
sets the flag even If the original value was not changed.