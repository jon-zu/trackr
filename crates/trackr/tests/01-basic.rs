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
        pub d: u32,
    }

    impl A {
        pub fn proxy_b_set(&mut self) {
            self.b_mut().set("Hello".to_string());
        }
    }
}

mod other {
    use super::data::A;
    // a works, but is private
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
    assert_eq!(a.flags(), AFlags::a);
    a.proxy_b_set();
    a.c_mut().force_update(|c| c.push(1));
    assert_eq!(a.flags(), AFlags::all());
}
