use forthen_core::errors::*;
use forthen_core::object_factory::StringManager;
use forthen_core::objects::object::Object;
use forthen_core::objects::{callable::Callable, prelude::*};
use forthen_core::Opcode;
use forthen_core::State;

pub fn class(state: &mut State) -> Result<()> {
    state.add_native_parse_word("None", |state| {
        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::Push(Object::None));
        Ok(())
    });

    state.add_native_word("{}", "( -- t)", |state| state.push(Object::new_table()));

    state.add_native_word("set_metatable", "(t mt -- t')", |state| {
        match state.pop()? {
            Object::Dynamic(mt) => state.top_mut()?.set_meta(Some(mt)),
            Object::None => state.top_mut()?.set_meta(None),
            _ => Err(ErrorKind::TypeError("meta table must be a table".to_string()).into()),
        }
    });

    state.add_native_word("get_metatable", "(t -- t mt)", |state| {
        let mt = state.top_mut()?.get_meta();
        match mt {
            Some(mt) => state.push(Object::Dynamic(mt)),
            None => state.push(Object::None),
        }
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

    state.add_native_parse_word("get_attr_raw", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        let name = state.factory.get_string(name);

        let get_func = Callable::new_const(
            move |state| {
                let value = state
                    .top()?
                    .get_attr(&name)
                    .ok_or(ErrorKind::AttributeError(name.to_string()))?;
                state.push(value)
            },
            "(obj -- obj val)",
        );

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::call_direct(get_func));
        Ok(())
    });

    state.add_native_parse_word("get_attr", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        let name = state.factory.get_string(name);

        let get_func = Callable::new_const(
            move |state| {
                let this = state.pop()?;
                state.push(name.clone())?;
                this.get_attribute(state)?;
                state.push(this)?;
                state.swap()
            },
            "(obj -- obj val)",
        );

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::call_direct(get_func));
        Ok(())
    });

    state.add_native_word("get_attribute", "(t s -- t v)", |state| {
        let attr = state.pop()?;
        let this = state.pop()?;
        state.push(attr)?;
        this.get_attribute(state)?;
        state.push(this)?;
        state.swap()
    });

    state.add_native_parse_word("has_attr", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        let name = state.factory.get_string(name);

        let get_func = Callable::new_const(
            move |state| match state.top()?.get_attr(&name) {
                Some(_) => state.push(Object::True),
                None => state.push(Object::False),
            },
            "(obj -- obj ?)",
        );

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::call_direct(get_func));
        Ok(())
    });

    // todo: this version of the Complex word creates a new table each time it is called.
    //       we need a way to define words that push a copy of the same value to the stack.
    state.run(
        "
        : class {} ;

        : Complex class
            [
                {}
                swap set_metatable
                1 set_attr real
                2 set_attr imag
            ] set_attr new

            [
                swap
                get_metatable
                rot
                get_attribute
                swap drop
            ] set_attr __index__

            [
                get_attr real
                swap
                get_attr imag
                rot swap
            ] set_attr get

            [
                get_attr get call
                rot drop
                rot
                get_attr get call
                rot drop

                rot +
                rot rot +
                swap
            ] set_attr __add__
        ;
    ",
    )?;

    Ok(())
}
