use forthen_core::errors::*;
use forthen_core::object_factory::StringManager;
use forthen_core::objects::{callable::Callable, dynobj::DynamicObjectImpl, prelude::*};
use forthen_core::Object;
use forthen_core::Opcode;
use forthen_core::State;
use std::rc::Rc;

/// Load basic operations into the dictionary
pub fn ops(state: &mut State) -> Result<()> {
    // math operations

    state.add_native_word("+", "( a b -- sum )", |state| {
        let b = state.pop()?;
        b.add(state)
    });

    state.add_native_word("-", "( a b -- diff )", |state| {
        let b = state.pop()?;
        let a = state.pop()?;
        state.push((a - b)?)
    });

    state.add_native_word("*", "( a b -- prod )", |state| {
        let b = state.pop()?;
        let a = state.pop()?;
        state.push((a * b)?)
    });

    state.add_native_word("/", "( a b -- quot )", |state| {
        let b = state.pop()?;
        let a = state.pop()?;
        state.push((a / b)?)
    });

    state.add_native_word("class", "( -- obj)", |state| {
        state.push(Object::Dynamic(Rc::new(DynamicObjectImpl::new())))
    });

    state.add_native_parse_word("set_attr", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        let name = state.factory.get_string(name);

        let set_func = Callable::new_const(
            move |state| {
                let value = state.pop()?;
                state.top_mut()?.set_attr(name.clone(), value);
                Ok(())
            },
            "(obj val -- obj')",
        );

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::call_direct(set_func));
        Ok(())
    });

    state.add_native_parse_word("get_attr", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        let name = state.factory.get_string(name);

        let get_func = Callable::new_const(
            move |state| {
                let value = state.top()?.get_attr(&name).ok_or(ErrorKind::AttributeError(name.to_string()))?;
                state.push(value)
            },
            "(obj -- obj val)",
        );

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::call_direct(get_func));
        Ok(())
    });

    Ok(())
}
