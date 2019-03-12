mod astack;
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

    macro_rules! assert_not_equivalent {
        ($a:expr, $b:expr) => {
            assert!(!$a.unwrap().is_equivalent(&$b.unwrap()))
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
        assert_not_equivalent!(
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
        assert_not_equivalent!(
            StackEffect::parse("(a b -- b a b a)"),
            StackEffect::parse("(x -- y)")
        );
        
        assert_equivalent!(
            StackEffect::parse("(..a f(..a -- ..b) -- ..b)"),
            StackEffect::parse("(..x g(..x -- ..y) -- ..y)")
        );
        
        assert_not_equivalent!(
            StackEffect::parse("(..a f(..a z -- ..b) -- ..b)"),
            StackEffect::parse("(..x g(..x -- z ..y) -- ..y)")
        );
        
        assert_equivalent!(
            StackEffect::parse("(f( f -- ) -- f )"),
            StackEffect::parse("(g( g -- ) -- g )")
        );
        
        assert_equivalent!(
            StackEffect::parse("(f( f -- x( -- ) ) -- x )"),
            StackEffect::parse("(g( g -- y( -- ) ) -- y )")
        );
        
        assert_not_equivalent!(
            StackEffect::parse("(f( f -- x( -- ) ) -- f )"),
            StackEffect::parse("(g( g -- y( -- ) ) -- y )")
        );
        
        assert_not_equivalent!(
            StackEffect::parse("(f( f -- x( a -- a a ) ) -- x )"),
            StackEffect::parse("(g( g -- y( a -- a b ) ) -- y )")
        );
    }
}