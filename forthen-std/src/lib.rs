mod class;
mod complex;
mod ops;
mod tier0;
mod tier1;

pub use class::class;
pub use complex::complex;
pub use ops::ops;
pub use tier0::tier0;
pub use tier1::tier1;

#[cfg(test)]
mod tests {
    use super::*;
    use forthen_core::State;

    #[test]
    fn recursion() {
        let mut state = State::new();
        tier0(&mut state).unwrap();
        tier1(&mut state).unwrap();
        ops(&mut state).unwrap();
        state.run(": recursive [ 1 - dup 0 == [ dup . drop drop ] [ dup . swap dup rot swap call ] if ] dup rot swap call ;").unwrap();
        state.run("10 recursive").unwrap();
    }
}
