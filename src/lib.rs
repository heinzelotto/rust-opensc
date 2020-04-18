#![allow(dead_code)]

#[macro_use]
extern crate anyhow;

use opensc_sys as ffi;
use std::marker::PhantomData;

trait RawWrapper<'a, S, O> {
    unsafe fn from_raw(_: &'a O, s: S) -> Self;

    fn as_raw(&self) -> S;
}

pub struct Card<'a, 'b> {
    cd: *mut ffi::sc_card_t,
    phantom: PhantomData<&'a Reader<'b>>,
}

impl<'a, 'b> RawWrapper<'a, *mut ffi::sc_card_t, Reader<'b>> for Card<'a, 'b> {
    unsafe fn from_raw(_: &'a Reader, cd: *mut ffi::sc_card_t) -> Card<'a, 'b> {
        Self {
            cd,
            phantom: PhantomData,
        }
    }

    fn as_raw(&self) -> *mut ffi::sc_card_t {
        self.cd
    }
}

impl Drop for Card<'_, '_> {
    // todo ?also reset card
    fn drop(&mut self) {
        unsafe {
            ffi::sc_disconnect_card(self.cd);
        }
    }
}

pub struct Reader<'a> {
    rd: *mut ffi::sc_reader_t,
    phantom: PhantomData<&'a Context>,
}

impl<'a> Reader<'a> {
    fn connect_card<'card>(&'a self) -> anyhow::Result<Card<'card, 'a>> {
        unsafe {
            let mut cd: *mut ffi::sc_card_t = std::ptr::null_mut();
            let r = ffi::sc_connect_card(self.rd, &mut cd);
            if r < 0 || cd == std::ptr::null_mut() {
                let err_str = std::ffi::CStr::from_ptr(ffi::sc_strerror(r))
                    .to_string_lossy()
                    .into_owned();
                return Err(anyhow!("Failed to connect to card: {}", err_str));
            }
            Ok(Card::from_raw(&self, cd))}
    }
}

impl<'a> RawWrapper<'a, *mut ffi::sc_reader_t, Context> for Reader<'a> {
    unsafe fn from_raw(_: &'a Context, rd: *mut ffi::sc_reader_t) -> Reader<'a> {
        Self {
            rd,
            phantom: PhantomData,
        }
    }

    fn as_raw(&self) -> *mut ffi::sc_reader_t {
        self.rd
    }
}

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

    /// Use default card driver with context.
    pub fn use_default_driver(&self) {
        unsafe {
            (*self.ctx).flags |= ffi::SC_CTX_FLAG_ENABLE_DEFAULT_DRIVER as u64;
        }
    }

    pub fn get_reader_count(&self) -> u32 {
        unsafe { ffi::sc_ctx_get_reader_count(self.ctx) }
    }

    pub fn get_reader_by_id<'a>(&'a self, id: u32) -> anyhow::Result<Reader<'a>> {
        unsafe {
            let raw_reader = ffi::sc_ctx_get_reader_by_id(self.ctx, id);
            if raw_reader.is_null() {
                return Err(anyhow!("Reader {} unavailable", id));
            }

            Ok(Reader::from_raw(&self, raw_reader))
        }
    }

    pub fn get_readers(&self) -> anyhow::Result<Vec<Reader>> {
        (0..self.get_reader_count())
            .map(|i| self.get_reader_by_id(i))
            .collect()
    }

    /// Access to the wrapped OpenSC raw context
    pub unsafe fn inner(&self) -> *mut ffi::sc_context_t {
        return self.ctx;
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            ffi::sc_release_context(self.ctx);
        }
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
        ctx.use_default_driver();
    }

    // works better with connected reader
    #[test]
    fn get_readers() {
        let ctx = Context::new().expect("context created");
        ctx.use_default_driver();
        if ctx.get_reader_count() > 0 {
            let _rd = ctx.get_reader_by_id(0).expect("reader #0 found");
        }

        let _rds = ctx.get_readers().expect("retrieved available readers");
    }

    // works better with connected reader and card
    #[test]
    fn get_card() {
        let ctx = Context::new().expect("context created");
        ctx.use_default_driver();
        if ctx.get_reader_count() > 0 {
            let rd = ctx.get_reader_by_id(0).expect("reader #0 found");

            // only works if card is present
            let _cd = rd.connect_card().expect("card connected");
        }
    }
}

