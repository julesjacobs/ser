//! A thin wrapper around the ISL C bindings, exposing just the operations we need
//!
//! A word on memory management: everything in ISL happens inside some ISL context. ISL contexts
//! are thread-local, and must outlive everything allocated using it.
//!
//! Instead of annotating everything with a `<'ctx>` lifetime, we'll just have a lazily-allocated
//! thread-local context that never gets freed.

use std::cell::Cell;
use std::ffi::{c_void, CString};
use std::{mem, ptr};

/// Based on https://rust-lang.github.io/rust-bindgen/tutorial-4.html
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unsafe_op_in_unsafe_fn)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/isl_bindings.rs"));
}
use bindings::*;

/// The lazily-allocated thread-local (but global within a thread) ISL context.
fn get_ctx() -> *mut isl_ctx {
    thread_local! {
        static CTX: Cell<*mut isl_ctx> = Cell::new(ptr::null_mut());
    }
    let mut ctx = CTX.get();
    if ctx.is_null() {
        ctx = unsafe { isl_ctx_alloc() };
        CTX.set(ctx);
    }
    ctx
}

/// Some functions return `isl_size`. This may be negative, in which case we panic.
fn check_isl_size(size: isl_size) -> usize {
    assert!(size >= 0);
    size as usize
}

/// An isl space. We only care about sets, so for us, the only thing that matters is the dimension.
///
/// (In particular, we only ever create set spaces with 0 parameters.)
///
/// Every set / basic set / etc. has a space; get it with `.space()`. Or, create a new space with
/// [`Space::new`].
#[derive(Debug)]
pub struct Space {
    ptr: *mut isl_space,
}

impl Drop for Space {
    fn drop(&mut self) {
        self.ptr = unsafe { isl_space_free(self.ptr) };
    }
}
impl Clone for Space {
    fn clone(&self) -> Self {
        Space {
            ptr: unsafe { isl_space_copy(self.ptr) },
        }
    }
}

impl Space {
    fn from_ptr(ptr: *mut isl_space) -> Self {
        assert!(!ptr.is_null(), "ptr is null!");
        Self { ptr }
    }
    fn into_ptr(self) -> *mut isl_space {
        let result = self.ptr;
        mem::forget(self);
        result
    }

    /// A new set space with the specified dimension
    pub fn new(dim: usize) -> Self {
        Self::from_ptr(unsafe { isl_space_set_alloc(get_ctx(), 0, dim as _) })
    }

    /// The dimension (number of variables) of the space
    pub fn dim(&self) -> usize {
        todo!()
        // check_isl_size(unsafe { isl_space_dim(self.ptr, isl_dim_set) })
    }
}

/// An isl basic set. A existentially-quantified conjunction of affine constraints
///
/// Turn it into a `Set` with `.into()`.
#[derive(Debug)]
pub struct BasicSet {
    ptr: *mut isl_basic_set,
}

impl Drop for BasicSet {
    fn drop(&mut self) {
        self.ptr = unsafe { isl_basic_set_free(self.ptr) };
    }
}
impl Clone for BasicSet {
    fn clone(&self) -> Self {
        BasicSet {
            ptr: unsafe { isl_basic_set_copy(self.ptr) },
        }
    }
}

impl BasicSet {
    fn into_ptr(self) -> *mut isl_basic_set {
        let result = self.ptr;
        mem::forget(self);
        result
    }
    fn from_ptr(ptr: *mut isl_basic_set) -> Self {
        assert!(!ptr.is_null(), "ptr is null!");
        BasicSet { ptr }
    }

    pub fn read_from_str(text: &str) -> Self {
        let text = CString::new(text).expect("text should not have ascii \\0 in it");
        BasicSet::from_ptr(unsafe { isl_basic_set_read_from_str(get_ctx(), text.as_ptr()) })
    }

    pub fn space(&self) -> Space {
        Space::from_ptr(unsafe { isl_basic_set_get_space(self.ptr) })
    }
}

/// An isl set. A union of basic sets
#[derive(Debug)]
pub struct Set {
    ptr: *mut isl_set,
}

impl Drop for Set {
    fn drop(&mut self) {
        self.ptr = unsafe { isl_set_free(self.ptr) };
    }
}
impl Clone for Set {
    fn clone(&self) -> Self {
        Set::from_ptr(unsafe { isl_set_copy(self.ptr) })
    }
}

impl Set {
    fn into_ptr(self) -> *mut isl_set {
        let result = self.ptr;
        mem::forget(self);
        result
    }
    fn from_ptr(ptr: *mut isl_set) -> Self {
        assert!(!ptr.is_null(), "ptr is null!");
        Set { ptr }
    }

    pub fn space(&self) -> Space {
        Space::from_ptr(unsafe { isl_set_get_space(self.ptr) })
    }

    pub fn union(self, other: Set) -> Set {
        Set::from_ptr(unsafe { isl_set_union(self.into_ptr(), other.into_ptr()) })
    }

    pub fn intersect(self, other: Set) -> Set {
        Set::from_ptr(unsafe { isl_set_intersect(self.into_ptr(), other.into_ptr()) })
    }

    pub fn subtract(self, other: Set) -> Set {
        Set::from_ptr(unsafe { isl_set_subtract(self.into_ptr(), other.into_ptr()) })
    }

    pub fn complement(self) -> Set {
        Set::from_ptr(unsafe { isl_set_complement(self.into_ptr()) })
    }

    pub fn read_from_str(text: &str) -> Self {
        let text = CString::new(text).expect("text should not have ascii \\0 in it");
        Set::from_ptr(unsafe { isl_set_read_from_str(get_ctx(), text.as_ptr()) })
    }

    /// Iterate over each basic set in the set. They may not be disjoint
    pub fn foreach_basic_set<F: FnMut(BasicSet)>(&self, mut f: F) {

        extern "C" fn callback<F: FnMut(BasicSet)>(
            bset: *mut isl_basic_set,
            user: *mut c_void,
        ) -> isl_stat {
            unsafe {
                let f: &mut F = &mut *(user as *mut F);
                f(BasicSet::from_ptr(bset));
                0 // isl_stat_ok
            }
        }

        unsafe {
            let user: *mut c_void = &mut f as *mut F as *mut c_void;
            isl_set_foreach_basic_set(self.ptr, Some(callback::<F>), user);
        }
    }
}

impl From<BasicSet> for Set {
    fn from(basic_set: BasicSet) -> Set {
        Set::from_ptr(unsafe { isl_set_from_basic_set(basic_set.into_ptr()) })
    }
}
