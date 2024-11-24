
use trackr::TrackedStruct;

mod data {
    use trackr::Tracked;

    #[derive(Tracked, Default)]
    pub struct A {
        #[track(flag)]
        tracker_flags: AFlags,
        #[track(pub_)]
        a: u8,
        b: String,
        pub c: Vec<usize>,
        #[track(skip)]
        pub _d: u32,
    }

    impl A {
        pub fn proxy_b_set(&mut self) {
            self.b_mut().set("Hello".to_string());
        }
    }
}

mod other {
    use super::data::A;
    // a is accesible through the field fn, but is private
    pub fn try_access_a(a: &mut A) {
        a.a_mut().set(2);
        assert_eq!(*a.a(), 2);
    }
}

use data::{AFlags, A};
fn main() {
    let mut a = A::default();

    assert!(a.flags().is_empty());
    other::try_access_a(&mut a);
    *a.a_mut() += 2;
    assert_eq!(a.a(), &4);
    a.a_mut().update_opt(|a| a.checked_sub(5));
    assert_eq!(a.a(), &4);
    assert_eq!(a.flags(), AFlags::a);
    a.proxy_b_set();
    a.c_mut().force_update(|c| c.push(1));
    assert_eq!(a.flags(), AFlags::all());


    let updates = a.take_updates();
    assert_eq!(updates, Some(AFlags::a | AFlags::b | AFlags::c));
    assert!(a.take_updates().is_none());
}
