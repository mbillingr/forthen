mod branch;
mod complex;
mod loops;
mod ops;
mod scope;
mod stack;
mod stdlib;
mod table;
mod tier0;
mod timeit;

pub use branch::branch;
pub use complex::complex;
pub use loops::loops;
pub use ops::ops;
pub use scope::scope;
pub use stack::stack;
pub use stdlib::stdlib;
pub use table::table;
pub use tier0::tier0;
pub use timeit::timeit;

#[cfg(test)]
mod tests {
    use super::*;
    use forthen_core::State;

    #[test]
    fn recursion() {
        let mut state = State::new();
        tier0(&mut state).unwrap();
        branch(&mut state).unwrap();
        scope(&mut state).unwrap();
        stack(&mut state).unwrap();
        ops(&mut state).unwrap();
        state.run("USE branch:").unwrap();
        state.run("USE ops:").unwrap();
        state.run("USE scope:").unwrap();
        state.run("USE stack:").unwrap();
        state
            .run(
                "
                    : stash (y x -- x y x) dup rot swap ;

                    :: factorial   (x -- x)
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
