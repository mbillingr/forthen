use forthen_core::errors::*;
use forthen_core::State;
use forthen_core::{objects::prelude::*, Object};
use std::any::Any;
use std::rc::Rc;

struct Complex {
    real: f64,
    imag: f64,
}

impl Complex {
    fn new(real: f64, imag: f64) -> Self {
        Complex { real, imag }
    }
}

impl ObjectInterface for Complex {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr_sys(&self) -> String {
        format!("({} + {}i)", self.real, self.imag)
    }

    fn cmp_equal(&self, state: &mut State) -> Result<()> {
        let other = state.pop()?;
        let result = match other.as_any().downcast_ref::<Self>() {
            Some(c) => self.real == c.real && self.imag == c.imag,
            None => false,
        };
        state.push(result)
    }

    fn as_number(&self) -> Option<&dyn NumberInterface> {
        Some(self)
    }
}

impl NumberInterface for Complex {
    fn add(&self, state: &mut State) -> Result<()> {
        let other = state.pop()?;
        let result = match other.as_any().downcast_ref::<Self>() {
            Some(c) => Complex::new(self.real + c.real, self.imag + c.imag),
            None => panic!("expected complex"),
        };
        state.push(result)
    }
}

impl From<Complex> for Object {
    fn from(c: Complex) -> Self {
        Object::Extension(Rc::new(c))
    }
}

/// Load basic operations into the dictionary
pub fn complex(state: &mut State) -> Result<()> {
    // math operations

    state.add_native_word("c_zero", "( -- x)", |state| {
        state.push(Complex::new(0.0, 0.0))
    });
    state.add_native_word("c_one", "( -- x)", |state| {
        state.push(Complex::new(1.0, 0.0))
    });
    state.add_native_word("c_i", "( -- x)", |state| state.push(Complex::new(0.0, 1.0)));

    state.add_native_word("c_add", "(a b -- c)", |state| {
        let a = state.pop()?;
        let b = state.pop()?;
        match (a, b) {
            (Object::Extension(a), Object::Extension(b)) => {
                let a = a.as_any().downcast_ref::<Complex>().unwrap();
                let b = b.as_any().downcast_ref::<Complex>().unwrap();
                state.push(Complex::new(a.real + b.real, a.imag + b.imag))
            }
            _ => panic!("expected complex"),
        }
    });

    Ok(())
}
