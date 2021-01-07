use std::prelude::v1::*;
use std::slice::*;
use std::str::from_utf8_unchecked;

#[repr(transparent)]
pub struct ReprCStr(*mut i8);

unsafe impl Send for ReprCStr {}

unsafe fn string_size(mut ptr: *const i8) -> usize {
    (1..)
        .take_while(|_| {
            let ret = *ptr;
            ptr = ptr.offset(1);
            ret != 0
        })
        .last()
        .unwrap_or(0)
        + 1
}

impl From<&str> for ReprCStr {
    fn from(from: &str) -> Self {
        let b = from
            .bytes()
            .take_while(|&b| b != 0)
            .chain(Some(0))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self(Box::leak(b).as_mut_ptr() as *mut _)
    }
}

impl From<String> for ReprCStr {
    fn from(from: String) -> Self {
        from.as_str().into()
    }
}

impl AsRef<str> for ReprCStr {
    fn as_ref(&self) -> &str {
        unsafe { from_utf8_unchecked(from_raw_parts(self.0 as *const _, string_size(self.0) - 1)) }
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
        let _ = unsafe { Box::from_raw(from_raw_parts_mut(self.0 as *mut _, string_size(self.0))) };
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

#[cfg(test)]
mod tests {
    use super::ReprCStr;
    #[test]
    fn string_size_matches() {
        assert_eq!(0, ReprCStr::from("").as_ref().len());
        assert_eq!(1, ReprCStr::from("1").as_ref().len());
        assert_eq!(5, ReprCStr::from("12345").as_ref().len());
    }
}
