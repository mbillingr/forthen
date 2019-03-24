use forthen_core::errors::*;
use forthen_core::objects::object::Object;
use forthen_core::objects::prelude::*;
use forthen_core::State;
use forthen_core::object_factory::StringManager;
use std::rc::Rc;

/// Load basic operations into the dictionary
pub fn list(state: &mut State) -> Result<()> {
    state.new_mod("list".to_string())?;

    state.add_native_word("str-to-list", "(s -- l)", |state| {
        let s = state.pop_string()?;
        let mut list = vec![];

        for ch in s.chars() {
            let obj = state.factory.get_string(ch.to_string());
            list.push(Object::String(obj));
        }

        state.push(Object::List(Rc::new(list)))
    });

    state.add_native_word("list-make", "(* n -- l)", |state| {
        let n = state.pop_i32()?;
        let mut list = vec![];
        for _ in 0..n {
            list.push(state.pop()?);
        }
        list.reverse();
        state.push(Object::List(Rc::new(list)))
    });

    state.add_native_word("list-destructure", "(l -- * n)", |state| {
        let rclist = state.pop()?.into_rc_vec()?;
        let n = rclist.len();
        match Rc::try_unwrap(rclist) {
            Ok(mut list) => {
                for x in list.drain(..) {
                    state.push(x)?;
                }
            }
            Err(rclist) => {
                for x in rclist.iter() {
                    state.push(x.clone())?;
                }
            }
        }
        state.push(n as i32)?;
        Ok(())
    });

    state.add_native_word("list-reverse", "(l -- l')", |state|{
        let list = state.top_mut()?.as_vec_mut()?;
        list.reverse();
        Ok(())
    });

    state.add_native_word("list-empty?", "(l -- l ?)", |state|{
        let empty = state.top()?.as_slice()?.is_empty();
        state.push(empty)
    });

    state.add_native_word("list-len", "(l -- l n)", |state|{
        let l = state.top()?.as_slice()?.len();
        state.push(l as i32)
    });

    state.add_native_word("list-get", "(l i -- l x)", |state| {
        let i = state.pop_i32()? as usize;
        let x = {
            let list = state.top()?.as_slice()?;
            list.get(i)
                .ok_or_else(|| ErrorKind::IndexError(i, list.len()))?
                .clone()
        };
        state.push(x)
    });

    state.add_native_word("list-set", "(l i x -- l')", |state| {
        let i = state.pop_i32()? as usize;
        let x = state.pop()?;
        let list = state.top_mut()?.as_vec_mut()?;
        let len = list.len();
        *list
            .get_mut(i)
            .ok_or_else(|| ErrorKind::IndexError(i, len))? = x;
        Ok(())
    });

    state.add_native_word("pop-back", "(l -- l' x)", |state| {
        let list = state.top_mut()?.as_vec_mut()?;
        let x = list.pop().ok_or_else(|| ErrorKind::IndexError(0, 0))?;
        state.push(x)
    });

    state.add_native_word("pop-front", "(l -- l' x)", |state| {
        let list = state.top_mut()?.as_vec_mut()?;
        if list.is_empty() {
            return Err(ErrorKind::IndexError(0, 0).into());
        }
        let x = list.remove(0);
        state.push(x)
    });

    state.add_native_word("push-back", "(l x -- l')", |state| {
        let x = state.pop()?;
        let list = state.top_mut()?.as_vec_mut()?;
        list.push(x);
        Ok(())
    });

    state.add_native_word("push-front", "(l x -- l')", |state| {
        let x = state.pop()?;
        let list = state.top_mut()?.as_vec_mut()?;
        list.insert(0, x);
        Ok(())
    });

    state.exit_mod().unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
