use crate::narra_instance::NarraInstance;
/// Newtype wrapping a reference (pointer) cast into 'usize'
/// together with a unique ID for protection.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct InstanceHandle(usize, i64);

/// Create handle from reference
impl From<&mut NarraInstance> for InstanceHandle {
    fn from(instance: &mut NarraInstance) -> Self {
        Self::new(instance)
    }
}

/// Recover reference from handle
impl AsMut<NarraInstance> for InstanceHandle {
    fn as_mut(&mut self) -> &mut NarraInstance {
        unsafe { std::mem::transmute(self.0) }
    }
}

impl InstanceHandle {
    /// Create handle from reference, using a random number as unique ID
    pub fn new(world: &mut NarraInstance) -> Self {
        let handle = unsafe { std::mem::transmute(world) };
        let unique_id = rand::random();

        Self(handle, unique_id)
    }

    /// Get the unique ID of this instance
    pub fn unique_id(&self) -> i64 {
        self.1
    }
}

use rhai::plugin::*;
/// API for handle to 'World'
#[export_module]
pub mod handle_module {
    use rhai::NativeCallContext;

    use super::InstanceHandle;

    pub type Handle = InstanceHandle;

    /// Draw a bunch of pretty shapes.
    #[rhai_fn(return_raw)]
    pub fn jump(
        context: NativeCallContext,
        handle: &mut Handle,
        jump_to: String,
    ) -> Result<(), Box<EvalAltResult>> {
        // Double check the pointer is still fresh
        // by comparing the handle's unique ID with
        // the version stored in the engine's tag!
        // if handle.unique_id() != context.tag().unwrap().as_int().unwrap() {
        //     return "Ouch! The handle is stale!".into();
        // }

        // Get the reference to 'World'
        let ninst: &mut NarraInstance = handle.as_mut();

        // ... work with reference
        ninst.perform_jump(jump_to);

        Ok(())
    }
}

pub fn register_narra_extern(engine: &mut rhai::Engine) {
    engine.register_global_module(exported_module!(handle_module).into());
}
