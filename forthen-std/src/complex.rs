use forthen_core::Result;
use forthen_core::State;
use forthen_core::{DynamicObject, Object};
use std::rc::Rc;
use std::any::Any;

struct Complex {
    real: f64,
    imag: f64,
}

impl Complex {
    fn new(real: f64, imag:f64) -> Self {
        Complex {
            real, imag
        }
    }
}

impl DynamicObject for Complex {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self ) -> String {
        format!("({} + {}i)", self.real, self.imag)
    }

    fn eq(&self, other: &dyn DynamicObject) -> bool {
        match other.as_any().downcast_ref::<Self>() {
            Some(c) => self.real == c.real && self.imag == c.imag,
            None => false,
        }
    }
}

/// Load basic operations into the dictionary
pub fn complex(state: &mut State) -> Result<()> {
    // math operations

    state.add_native_word("c_zero", "( -- x)", |state| state.push(Object::Dynamic(Rc::new(Complex::new(0.0, 0.0)))));
    state.add_native_word("c_one", "( -- x)", |state| state.push(Object::Dynamic(Rc::new(Complex::new(1.0, 0.0)))));
    state.add_native_word("c_i", "( -- x)", |state| state.push(Object::Dynamic(Rc::new(Complex::new(0.0, 1.0)))));

    state.add_native_word("c_add", "(a b -- c)", |state| {
        let a = state.pop()?;
        let b = state.pop()?;
        match (a, b) {
            (Object::Dynamic(a), Object::Dynamic(b)) => {
                let a = a.as_any().downcast_ref::<Complex>().unwrap();
                let b = b.as_any().downcast_ref::<Complex>().unwrap();
                state.push(Object::Dynamic(Rc::new(Complex::new(a.real + b.real, a.imag + b.imag))))
            }
            _ => panic!("expected complex")
        }
    });

    Ok(())
}
