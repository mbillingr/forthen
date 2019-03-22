use std::fmt::Debug;

use crate::objects::Object;
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

    pub fn assert_pop<T>(&mut self, expected: T)
    where
        Object: std::cmp::PartialEq<T>,
        T: Debug,
    {
        match self.stack.pop() {
            Some(item) => assert_eq!(item, expected),
            None => panic!("tried to pop {:?} from empty stack", expected),
        }
    }

    pub fn assert_run_pop<T>(&mut self, cmd: &str, expected: &[T])
    where
        Object: std::cmp::PartialEq<T>,
        T: Debug,
    {
        self.run(cmd).unwrap();

        let mut stack_top: Vec<_> = (0..expected.len())
            .filter_map(|_| self.stack.pop())
            .collect();
        stack_top.reverse();

        if stack_top.len() != expected.len() || stack_top.iter().zip(expected).any(|(s, e)| s != e)
        {
            panic!(
                "Expected {:?} on top of stack but found {:?}",
                expected, stack_top
            );
        }
    }
}
