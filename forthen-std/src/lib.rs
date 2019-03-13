mod class;
mod complex;
mod ops;
mod scope;
mod tier0;
mod tier1;

pub use class::class;
pub use complex::complex;
pub use ops::ops;
pub use scope::scope;
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
        scope(&mut state).unwrap();
        tier1(&mut state).unwrap();
        ops(&mut state).unwrap();
        state.run("USE scope:").unwrap();
        state.run("USE tier1:").unwrap();
        state.run("USE ops:").unwrap();
        state
            .run(
                "
                    : stash dup rot swap ;

                    :: factorial
                    1 +
                    1 set acc
                    [
                        1 - dup 0 ==
                        [ drop drop ]
                        [
                            dup get acc * set acc
                            swap stash call
                        ]
                        if
                    ]
                    stash call
                    get acc
                ;",
            )
            .unwrap();
        state.run("\"guard\" 10 factorial").unwrap();

        assert_eq!(3628800, state.pop_i32().unwrap());
        assert_eq!("guard", state.pop_string().unwrap());
    }
}
