use dataview::Pod;
use std::str;

#[derive(Clone, Pod)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Win32OffsetsFile {
    // Win32GUID
    pub pdb_file_name: BinaryString,
    pub pdb_guid: BinaryString,

    // Win32Version
    pub nt_major_version: u32,
    pub nt_minor_version: u32,
    pub nt_build_number: u32,

    pub offsets: Win32OffsetsData,
}

// TODO: use const-generics here once they are fully stabilized
#[derive(Clone)]
pub struct BinaryString([u8; 128]);

// TODO: add from/to string/str methods

unsafe impl Pod for BinaryString {}

#[cfg(feature = "serde")]
impl ::serde::Serialize for BinaryString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let len = self.0[0] as usize;
        let string = str::from_utf8(&self.0[1..len]).unwrap();
        serializer.serialize_str(string)
    }
}

#[cfg(feature = "serde")]
impl<'de> ::serde::de::Deserialize<'de> for BinaryString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::de::Deserializer<'de>,
    {
        struct BinaryStringVisitor;

        impl<'de> ::serde::de::Visitor<'de> for BinaryStringVisitor {
            type Value = [u8; 128];

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string containing json data")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                // unfortunately we lose some typed information
                // from errors deserializing the json string
                let mut result = [0u8; 128];

                result[0] = v.len() as u8;
                result[1..v.len() + 1].copy_from_slice(v.as_bytes());

                Ok(result)
            }
        }

        // use our visitor to deserialize an `ActualValue`
        let inner: [u8; 128] = deserializer.deserialize_any(BinaryStringVisitor)?;
        Ok(Self { 0: inner })
    }
}

#[derive(Debug, Clone, Pod)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Win32OffsetsData {
    pub list_blink: usize,
    pub eproc_link: usize,

    pub kproc_dtb: usize,
    pub eproc_pid: usize,
    pub eproc_name: usize,
    pub eproc_peb: usize,
    pub eproc_thread_list: usize,
    pub eproc_wow64: usize,

    pub kthread_teb: usize,
    pub ethread_list_entry: usize,
    pub teb_peb: usize,
    pub teb_peb_x86: usize,
}
