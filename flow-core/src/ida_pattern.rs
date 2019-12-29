use crate::error::{Error, Result};
use regex::bytes::*;

// converts an ida pattern into a regex expression and tries to compile
fn ida_pattern_to_regex(pattern: &str) -> Result<Regex> {
    let regex = pattern
        .to_string()
        .split_whitespace()
        .map(|p| match p {
            "?" | "??" => "(?s:.)".to_owned(),
            "*" | "**" => "(?s:.*)".to_owned(),
            p => {
                if p.len() == 1 {
                    format!("\\x0{}", p)
                } else {
                    format!("\\x{}", p)
                }
            }
        })
        .collect::<Vec<String>>()
        .join("");

    Ok(Regex::new(&format!("(?-u){}", regex))?)
}

// String and &str helper implementations
pub trait IDARegex {
    fn try_as_ida_regex(&self) -> Result<Regex>;

    fn try_match_ida_regex(&self, haystack: &[u8]) -> Result<(usize, usize)> {
        let re = self.try_as_ida_regex()?;
        let m = re
            .find(haystack)
            .ok_or_else(|| Error::new("needle not found haystack"))?;
        Ok((m.start(), m.end()))
    }
}

impl IDARegex for String {
    fn try_as_ida_regex(&self) -> Result<Regex> {
        ida_pattern_to_regex(self.as_str())
    }
}

impl IDARegex for &str {
    fn try_as_ida_regex(&self) -> Result<Regex> {
        ida_pattern_to_regex(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_pattern() {
        let re = ida_pattern_to_regex("74 65 73 74 31 32 33 34 74 65 73 74").unwrap();
        assert_eq!(re.find(b"testtest1234test").unwrap().start(), 4);
    }

    #[test]
    fn test_basic_single_wildcard() {
        let re = ida_pattern_to_regex("74 65 73 74 74 65 73 74 ? ?? ? ?? 74 65 73 74").unwrap();
        assert_eq!(re.find(b"testtest1234test").unwrap().start(), 0);
    }

    #[test]
    fn test_basic_multi_wildcard() {
        let re = ida_pattern_to_regex("74 65 73 74 74 65 73 74 * ** 74 65 73 74").unwrap();
        assert_eq!(re.find(b"testtest1234321test").unwrap().start(), 0);
    }

    #[test]
    fn test_basic_nonascii() {
        let re = ida_pattern_to_regex("74 65 73 74 FF 31 32 0A 34 74 65 73 74").unwrap();
        assert_eq!(re.find(b"testtest\xFF12\n4test").unwrap().start(), 4);
    }

    #[test]
    fn test_basic_nonascii_single_char() {
        let re = ida_pattern_to_regex("74 65 73 74 FF 31 32 A 34 74 65 73 74").unwrap();
        assert_eq!(re.find(b"testtest\xFF12\n4test").unwrap().start(), 4);
    }

    #[test]
    fn test_nonascii_single_wildcard() {
        let re = ida_pattern_to_regex("74 65 73 74 FF 74 65 73 74 ? ?? ? ?? 74 65 73 74").unwrap();
        assert_eq!(re.find(b"test\xFFtest12\n4test").unwrap().start(), 0);
    }

    #[test]
    fn test_nonascii_multi_wildcard() {
        let re = ida_pattern_to_regex("74 65 73 74 FF 74 65 73 74 ** * 74 65 73 74").unwrap();
        assert_eq!(re.find(b"test\xFFtest12\n4test").unwrap().start(), 0);
    }
}
