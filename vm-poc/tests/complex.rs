use vm_poc::*;

use complex::Complex;

#[test]
fn it_works() {
    let mut state = State::default();

    let id = state.deploy(Complex::default());

    assert_eq!(state.query::<_, u32>(id, "check_hair", ()).unwrap(), 0)
}
