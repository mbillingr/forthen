use std::fmt::Debug;

use crate::Object;
use crate::State;

impl State {
    pub fn assert_stack<T>(&self, expected: &[T])
    where
        Object: std::cmp::PartialEq<T>,
        T: Debug,
    {
        assert_eq!(self.stack, expected)
    }

    pub fn assert_stack_top<T>(&self, expected: &[T])
    where
        Object: std::cmp::PartialEq<T>,
        T: Debug,
    {
        for (s, e) in self.stack.iter().rev().zip(expected.iter().rev()) {
            assert_eq!(s, e)
        }
    }
}
