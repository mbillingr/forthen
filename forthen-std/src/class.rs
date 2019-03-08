use forthen_core::errors::*;
use forthen_core::State;
use forthen_core::objects::{callable::Callable, dynobj::DynamicObjectImpl, prelude::*};
use forthen_core::Opcode;
use forthen_core::object_factory::StringManager;
use std::rc::Rc;
use forthen_core::objects::object::Object;


pub fn class(state: &mut State) -> Result<()> {

    state.add_native_parse_word("None", |state| {
        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::Push(Object::None));
        Ok(())
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

    state.add_native_parse_word("has_attr", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        let name = state.factory.get_string(name);

        let get_func = Callable::new_const(
            move |state| {
                match state.top()?.get_attr(&name) {
                    Some(_) => state.push(Object::True),
                    None => state.push(Object::False),
                }
            },
            "(obj -- obj ?)",
        );

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::call_direct(get_func));
        Ok(())
    });

    /*state.add_native_parse_word("call_method", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        let name = state.factory.get_string(name);

        let func = Callable::new_const(
            move |state| {
                let value = state.top()?.get_attr(&name).ok_or(ErrorKind::AttributeError(name.to_string()))?;
                state.push(value)
            },
            "(obj -- obj val)",
        );

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::call_direct(func));
        Ok(())
    });*/

//    state.run(":: call_method next_token dup has_attr [ . ] [ . ] if ;")?;

    state.run("
        class
            \"Complex\" set_attr __name__
            None set_attr __class__

            [
                class
                swap set_attr __class__
            ] set_attr __call__

            [
                swap set_attr imag
                swap set_attr real
            ] set_attr __init__

    ")?;

    Ok(())
}
