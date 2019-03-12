mod comparison;
mod effect;
mod element;
mod parser;
mod scratchpad;
mod sequence;

pub use effect::StackEffect;
pub use parser::parse_effect;


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_equivalent {
        ($a:expr, $b:expr) => {
            assert!($a.unwrap().is_equivalent(&$b.unwrap()))
        };
    }

    #[test]
    fn equivalence_effects() {
        assert_equivalent!(
            StackEffect::parse("( -- )"),
            StackEffect::parse("(--)")
        );
        assert_equivalent!(
            StackEffect::parse("(b -- b)"),
            StackEffect::parse("(a -- a)")
        );
        assert_equivalent!(
            StackEffect::parse("(x y -- y x)"),
            StackEffect::parse("(a b -- b a)")
        );
        assert_equivalent!(
            StackEffect::parse("(a b -- a a)"),
            StackEffect::parse("(a b -- b b)")
        );
        assert_equivalent!(
            StackEffect::parse("(a b -- c)"),
            StackEffect::parse("(b a -- z)")
        );
        assert_equivalent!(
            StackEffect::parse("( -- a b)"),
            StackEffect::parse("( -- b a)")
        );
        assert_equivalent!(
            StackEffect::parse("(b -- a b b c)"),
            StackEffect::parse("(b -- c b b a)")
        );
        assert_equivalent!(
            StackEffect::parse("(a b -- b a b a)"),
            StackEffect::parse("(x -- y)")
        );
    }
}