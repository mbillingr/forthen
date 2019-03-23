use forthen_core::errors::*;
use forthen_core::object_factory::StringManager;
use forthen_core::objects::object::Object;
use forthen_core::objects::{callable::Callable, prelude::*};
use forthen_core::State;

pub fn table(state: &mut State) -> Result<()> {
    state.new_mod("table".to_string())?;

    state.add_native_word("{}", "( -- t)", |state| state.push(Object::new_table()));

    state.add_native_word("set_metatable", "(t mt -- t')", |state| {
        match state.pop()? {
            Object::Table(mt) => state.top_mut()?.set_meta(Some(mt)),
            Object::None => state.top_mut()?.set_meta(None),
            _ => Err(ErrorKind::TypeError("meta table must be a table".to_string()).into()),
        }
    });

    state.add_native_word("get_metatable", "(t -- t mt)", |state| {
        let mt = state.top_mut()?.get_meta();
        match mt {
            Some(mt) => state.push(Object::Table(mt)),
            None => state.push(Object::None),
        }
    });

    state.add_native_parse_word("set_attr", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        let name = state.factory.get_string(name);

        let set_func = Callable::new_const(move |state| {
            let value = state.pop()?;
            state.top_mut()?.set_attr(name.clone(), value);
            Ok(())
        });

        let instructions = state.top_mut()?.as_vec_mut()?;
        instructions.push(Object::Function(set_func));
        Ok(())
    });

    state.add_native_parse_word("get_attr_raw", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        let name = state.factory.get_string(name);

        let get_func = Callable::new_const(move |state| {
            let value = state
                .top()?
                .get_attr(&name)
                .ok_or_else(|| ErrorKind::AttributeError(name.to_string()))?;
            state.push(value)
        });

        let instructions = state.top_mut()?.as_vec_mut()?;
        instructions.push(Object::Function(get_func));
        Ok(())
    });

    state.add_native_parse_word("get_attr", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        let name = state.factory.get_string(name);

        let get_func = Callable::new_const(move |state| {
            let this = state.pop()?;
            state.push(name.clone())?;
            this.get_attribute(state)?;
            state.push(this)?;
            state.swap()
        });

        let instructions = state.top_mut()?.as_vec_mut()?;
        instructions.push(Object::Function(get_func));
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

        let get_func = Callable::new_const(move |state| match state.top()?.get_attr(&name) {
            Some(_) => state.push(Object::True),
            None => state.push(Object::False),
        });

        let instructions = state.top_mut()?.as_vec_mut()?;
        instructions.push(Object::Function(get_func));
        Ok(())
    });

    state.exit_mod().unwrap();

    Ok(())
}
