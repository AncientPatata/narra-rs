use crate::narra_instance::NarraInstance;

use rlua::{UserData, UserDataMethods};

pub struct NarraInstanceHandle(pub *mut NarraInstance);
unsafe impl Send for NarraInstanceHandle {}
impl UserData for NarraInstanceHandle {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("jump", |_, this, jump_to: String| {
            Ok(unsafe { this.0.as_mut().unwrap().perform_jump(jump_to) })
        });

        methods.add_method("seen", |_, this, action_id: String| {
            Ok(unsafe { this.0.as_mut().unwrap().state.seen_action(action_id) })
        });

        methods.add_method_mut("set_block", |_, this, val: bool| {
            Ok(unsafe { this.0.as_mut().unwrap().blocked = val })
        });

        methods.add_method("is_blocked", |_, this, def: bool| {
            Ok(unsafe { this.0.as_mut().unwrap().blocked })
        });
    }
}
