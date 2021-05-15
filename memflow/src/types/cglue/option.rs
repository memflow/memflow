/// Describes a FFI safe option
#[repr(C)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum COption<T> {
    None,
    Some(T),
}

impl<T> From<Option<T>> for COption<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            None => Self::None,
            Some(t) => Self::Some(t),
        }
    }
}

impl<T> From<COption<T>> for Option<T> {
    fn from(opt: COption<T>) -> Self {
        match opt {
            COption::None => None,
            COption::Some(t) => Some(t),
        }
    }
}

impl<T> COption<T> {
    pub const fn is_some(&self) -> bool {
        matches!(*self, COption::Some(_))
    }

    pub fn unwrap(self) -> T {
        match self {
            COption::Some(val) => val,
            COption::None => panic!("called `COption::unwrap()` on a `None` value"),
        }
    }

    pub const fn as_ref(&self) -> Option<&T> {
        match *self {
            COption::Some(ref x) => Some(x),
            COption::None => None,
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        match *self {
            COption::Some(ref mut x) => Some(x),
            COption::None => None,
        }
    }
}
