use crate::support::{Big, Fixture, Pair};
use amml_msvc::MemberCall;
use amml_msvc_fixture::{fx_thing_make_address, fx_thing_pair_address};

#[test]
fn by_value_member_return_uses_the_pointer_after_this() {
    let mut thing = Fixture::thing(3);
    // SAFETY: the address is `Big Thing::make(int)` and `thing` mirrors `Thing`.
    let big: Big = unsafe { MemberCall::call_sret1(fx_thing_make_address(), (&raw mut thing).cast(), 4i32) };
    assert_eq!((big.a, big.b, big.c), (3, 4, 12));
}

#[test]
fn small_trivially_copyable_member_return_still_uses_the_pointer_after_this() {
    let mut thing = Fixture::thing(3);
    // SAFETY: the address is `Pair Thing::pair(int)` and `thing` mirrors `Thing`.
    let pair: Pair = unsafe { MemberCall::call_sret1(fx_thing_pair_address(), (&raw mut thing).cast(), 4i32) };
    assert_eq!((pair.a, pair.b), (3, 4));
}
