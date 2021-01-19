/// FFI-Safe Arc
///
/// This arc is essentially a wrapper around `std::sync::Arc`. It depends on a lot of heavily
/// unsafe code, with a few assumptions taken into consideration:
///
/// 1. Arc<T> takes up one pointer of storage
/// 2. CArc, and COptArc have the same layout
///
/// #1 and part of #2 are checked in this module at compile time.
///
/// Even though this code should be safe to use, it is private for internal use only
use std::ops::Deref;
use std::sync::Arc;

unsafe extern "C" fn c_clone<T: Sized + 'static>(
    ptr_to_arc: *const &*const T,
) -> &'static *const T {
    let cloned_arc = (ptr_to_arc as *const Arc<T>).as_ref().unwrap().clone();
    std::mem::transmute(cloned_arc)
}

unsafe extern "C" fn c_drop<T: Sized + 'static>(ptr_to_arc: *mut &*const T) {
    std::ptr::drop_in_place(ptr_to_arc as *mut Arc<T>)
}

#[repr(C)]
pub struct CArc<T: Sized + 'static> {
    inner: &'static *const T,
    clone_fn: unsafe extern "C" fn(*const &*const T) -> &'static *const T,
    drop_fn: unsafe extern "C" fn(*mut &*const T),
}

impl<T> From<T> for CArc<T> {
    fn from(obj: T) -> Self {
        let inner = unsafe { std::mem::transmute(Some(Arc::new(obj))) };
        Self {
            inner,
            clone_fn: c_clone,
            drop_fn: c_drop,
        }
    }
}

unsafe impl<T: Sync + Send> Send for CArc<T> {}
unsafe impl<T: Sync + Send> Sync for CArc<T> {}

impl<T> Clone for CArc<T> {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { (self.clone_fn)(&self.inner) },
            ..*self
        }
    }
}

impl<T> Drop for CArc<T> {
    fn drop(&mut self) {
        println!("DROPPING ARC {:?}", self.inner as *const _);
        unsafe { (self.drop_fn)(&mut self.inner) }
    }
}

impl<T> Deref for CArc<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { (&self.inner as *const &*const T as *const Arc<T>).as_ref() }.unwrap()
    }
}

impl<T> AsRef<T> for CArc<T> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

unsafe impl<T: Sync + Send> Send for COptArc<T> {}
unsafe impl<T: Sync + Send> Sync for COptArc<T> {}

#[repr(C)]
pub struct COptArc<T: Sized + 'static> {
    inner: *const *const T,
    clone_fn: Option<unsafe extern "C" fn(*const &*const T) -> &'static *const T>,
    drop_fn: Option<unsafe extern "C" fn(*mut &*const T)>,
}

impl<T> Clone for COptArc<T> {
    fn clone(&self) -> Self {
        match <Option<&CArc<T>>>::from(self) {
            Some(arc) => Some(arc.clone()).into(),
            None => None.into(),
        }
    }
}

impl<T> Drop for COptArc<T> {
    fn drop(&mut self) {
        if let Some(arc) = <Option<&mut CArc<T>>>::from(self) {
            unsafe { std::ptr::drop_in_place(arc) };
        }
    }
}

impl<T> From<Option<CArc<T>>> for COptArc<T> {
    fn from(opt: Option<CArc<T>>) -> Self {
        match opt {
            Some(arc) => Self {
                inner: arc.inner,
                clone_fn: Some(arc.clone_fn),
                drop_fn: Some(arc.drop_fn),
            },
            None => Self {
                inner: std::ptr::null(),
                clone_fn: None,
                drop_fn: None,
            },
        }
    }
}

impl<T> From<&mut COptArc<T>> for Option<&mut CArc<T>> {
    fn from(copt: &mut COptArc<T>) -> Self {
        if copt.inner.is_null() {
            None
        } else {
            unsafe { (copt as *mut COptArc<T>).cast::<CArc<T>>().as_mut() }
        }
    }
}

impl<T> From<&COptArc<T>> for Option<&CArc<T>> {
    fn from(copt: &COptArc<T>) -> Self {
        if copt.inner.is_null() {
            None
        } else {
            unsafe { (copt as *const COptArc<T>).cast::<CArc<T>>().as_ref() }
        }
    }
}

impl<T> From<COptArc<T>> for Option<CArc<T>> {
    fn from(copt: COptArc<T>) -> Self {
        match copt {
            COptArc {
                inner,
                clone_fn: Some(clone_fn),
                drop_fn: Some(drop_fn),
            } => Some(CArc {
                inner: unsafe { inner.as_ref().unwrap() },
                clone_fn,
                drop_fn,
            }),
            _ => None,
        }
    }
}

// FFI depends on library option arc being null pointer optimizable
const _: [(); std::mem::size_of::<Arc<u128>>()] = [(); std::mem::size_of::<&'static usize>()];
const _: [(); std::mem::size_of::<Option<Arc<u128>>>()] =
    [(); std::mem::size_of::<Option<&'static usize>>()];
const _: [(); std::mem::size_of::<Option<Arc<u128>>>()] =
    [(); std::mem::size_of::<&'static usize>()];
const _: [(); std::mem::size_of::<CArc<u128>>()] = [(); std::mem::size_of::<COptArc<u128>>()];
