#![allow(dead_code)]

#[macro_use]
extern crate anyhow;


use opensc_sys as ffi;

pub struct Context {
    // currently unused
    _parms: ffi::sc_context_param_t,
    // the context object
    ctx: *mut ffi::sc_context_t,
}

impl Context {

    /// Create a new context with default params.
    pub fn new() -> anyhow::Result<Self> {
        unsafe {
            let parms = std::mem::zeroed();
            let mut ctx: *mut ffi::sc_context_t = std::ptr::null_mut();
            let r = ffi::sc_context_create(&mut ctx, &parms);
            if r < 0 || ctx == std::ptr::null_mut() {
                let err_str = std::ffi::CStr::from_ptr(ffi::sc_strerror(r))
                    .to_string_lossy()
                    .into_owned();
                return Err(anyhow!("Failed to create initial context: {}", err_str));
            }

            Ok(Self { _parms: parms, ctx })
        }
    }

    /// Access to the wrapped OpenSC raw context
    unsafe fn inner(&self) -> *mut ffi::sc_context_t {
        return self.ctx;
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ffi::sc_release_context(self.ctx); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_context() {
        let ctx = Context::new().expect("context created");
        let raw_ctx = unsafe { ctx.inner() };
        assert!(!raw_ctx.is_null());
    }
}

