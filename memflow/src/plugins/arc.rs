/// FFI-Safe Arc
///
/// This Arc essentially uses clone/drop from the module that created it, to not mix up global
/// allocators.
///
/// Even though this code should be safe to use, it is private for internal use only
use std::sync::Arc;

unsafe extern "C" fn c_clone<T: Sized + 'static>(
    ptr_to_arc: Option<&'static T>,
) -> Option<&'static T> {
    if let Some(p) = ptr_to_arc {
        let arc = Arc::from_raw(p);
        let cloned_arc = arc.clone();
        let _ = Arc::into_raw(arc);
        Arc::into_raw(cloned_arc).as_ref()
    } else {
        None
    }
}

unsafe extern "C" fn c_drop<T: Sized + 'static>(ptr_to_arc: &mut Option<&T>) {
    if let Some(p) = ptr_to_arc.take() {
        Arc::from_raw(p);
    }
}

#[repr(C)]
pub struct CArc<T: Sized + 'static> {
    inner: Option<&'static T>,
    clone_fn: unsafe extern "C" fn(Option<&'static T>) -> Option<&'static T>,
    drop_fn: unsafe extern "C" fn(&mut Option<&T>),
}

impl<T> From<T> for CArc<T> {
    fn from(obj: T) -> Self {
        let arc = Arc::new(obj);
        Self {
            inner: unsafe { Arc::into_raw(arc).as_ref() },
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
            inner: unsafe { (self.clone_fn)(self.inner) },
            ..*self
        }
    }
}

impl<T> Drop for CArc<T> {
    fn drop(&mut self) {
        unsafe { (self.drop_fn)(&mut self.inner) }
    }
}

impl<T> AsRef<T> for CArc<T> {
    fn as_ref(&self) -> &T {
        self.inner.unwrap()
    }
}

unsafe impl<T: Sync + Send> Send for COptArc<T> {}
unsafe impl<T: Sync + Send> Sync for COptArc<T> {}

#[repr(C)]
pub struct COptArc<T: Sized + 'static> {
    inner: Option<&'static T>,
    clone_fn: Option<unsafe extern "C" fn(Option<&'static T>) -> Option<&'static T>>,
    drop_fn: Option<unsafe extern "C" fn(&mut Option<&T>)>,
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

impl<T> COptArc<T> {
    pub fn take(&mut self) -> COptArc<T> {
        Self {
            inner: self.inner.take(),
            clone_fn: self.clone_fn.take(),
            drop_fn: self.drop_fn.take(),
        }
    }
}

impl<T> From<Option<CArc<T>>> for COptArc<T> {
    fn from(opt: Option<CArc<T>>) -> Self {
        match opt {
            Some(mut arc) => Self {
                inner: arc.inner.take(),
                clone_fn: Some(arc.clone_fn),
                drop_fn: Some(arc.drop_fn),
            },
            None => Self {
                inner: None,
                clone_fn: None,
                drop_fn: None,
            },
        }
    }
}

impl<T> From<&mut COptArc<T>> for Option<&mut CArc<T>> {
    fn from(copt: &mut COptArc<T>) -> Self {
        if copt.inner.is_none() {
            None
        } else {
            unsafe { (copt as *mut COptArc<T>).cast::<CArc<T>>().as_mut() }
        }
    }
}

impl<T> From<&COptArc<T>> for Option<&CArc<T>> {
    fn from(copt: &COptArc<T>) -> Self {
        if copt.inner.is_none() {
            None
        } else {
            unsafe { (copt as *const COptArc<T>).cast::<CArc<T>>().as_ref() }
        }
    }
}

impl<T> From<COptArc<T>> for Option<CArc<T>> {
    fn from(mut copt: COptArc<T>) -> Self {
        let ai = copt.inner.take();
        match copt {
            COptArc {
                inner: _,
                clone_fn: Some(clone_fn),
                drop_fn: Some(drop_fn),
            } => Some(CArc {
                inner: ai,
                clone_fn,
                drop_fn,
            }),
            _ => None,
        }
    }
}

const _: [(); std::mem::size_of::<CArc<u128>>()] = [(); std::mem::size_of::<COptArc<u128>>()];
