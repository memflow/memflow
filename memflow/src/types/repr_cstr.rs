use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(transparent)]
pub struct ReprCStr(*mut c_char);

unsafe impl Send for ReprCStr {}

impl From<&str> for ReprCStr {
    fn from(from: &str) -> Self {
        Self(CString::new(from).expect("CString::new failed").into_raw())
    }
}

impl From<String> for ReprCStr {
    fn from(from: String) -> Self {
        from.as_str().into()
    }
}

impl From<ReprCStr> for CString {
    fn from(from: ReprCStr) -> CString {
        unsafe { CString::from_raw(from.0) }
    }
}

impl AsRef<str> for ReprCStr {
    fn as_ref(&self) -> &str {
        unsafe { CStr::from_ptr(self.0) }.to_str().unwrap()
    }
}

impl std::ops::Deref for ReprCStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Drop for ReprCStr {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.0) };
    }
}

impl Clone for ReprCStr {
    fn clone(&self) -> Self {
        self.as_ref().into()
    }
}

impl std::fmt::Display for ReprCStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl std::fmt::Debug for ReprCStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReprCStr")
            .field("0", &self.as_ref())
            .finish()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for ReprCStr {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ReprCStr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<ReprCStr, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ReprCStrVisitor;

        impl<'de> ::serde::de::Visitor<'de> for ReprCStrVisitor {
            type Value = ReprCStr;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Ok(v.into())
            }
        }

        deserializer.deserialize_str(ReprCStrVisitor)
    }
}
