use crate::error::{Error, Result};

use core::convert::TryFrom;
use std::collections::HashMap;

/// Argument wrapper for connectors
///
/// # Examples
///
/// Construct from a string:
/// ```
/// use memflow_core::connector::ConnectorArgs;
/// use std::convert::TryFrom;
///
/// let argstr = "opt1=test1,opt2=test2,opt3=test3";
/// let args = ConnectorArgs::try_from(argstr).unwrap();
/// ```
///
/// Construct as builder:
/// ```
/// use memflow_core::connector::ConnectorArgs;
///
/// let args = ConnectorArgs::new()
///     .insert("arg1", "test1")
///     .insert("arg2", "test2");
/// ```
pub struct ConnectorArgs {
    map: HashMap<String, String>,
}

impl ConnectorArgs {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn with_default(value: &str) -> Self {
        Self::new().insert("default", value)
    }

    pub fn try_parse_str(args: &str) -> Result<Self> {
        let mut map = HashMap::new();

        // if args != "" {
        let split = args.split(',');
        for (i, kv) in split.clone().enumerate() {
            let kvsplit = kv.split('=').collect::<Vec<_>>();
            if kvsplit.len() == 2 {
                map.insert(kvsplit[0].to_string(), kvsplit[1].to_string());
            } else if i == 0 {
                map.insert("default".to_string(), kv.to_string());
            }
        }
        // }

        Ok(Self { map })
    }

    pub fn insert(mut self, key: &str, value: &str) -> Self {
        self.map.insert(key.to_string(), value.to_string());
        self
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }

    pub fn get_default(&self) -> Option<&String> {
        self.get("default")
    }
}

impl Default for ConnectorArgs {
    fn default() -> Self {
        ConnectorArgs::new()
    }
}

impl TryFrom<&str> for ConnectorArgs {
    type Error = Error;

    fn try_from(args: &str) -> Result<Self> {
        ConnectorArgs::try_parse_str(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn from_str() {
        let argstr = "opt1=test1,opt2=test2,opt3=test3";
        let args = ConnectorArgs::try_from(argstr).unwrap();
        assert_eq!(args.get("opt1").unwrap(), "test1");
        assert_eq!(args.get("opt2").unwrap(), "test2");
        assert_eq!(args.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn from_str_default() {
        let argstr = "test0,opt1=test1,opt2=test2,opt3=test3";
        let args = ConnectorArgs::try_from(argstr).unwrap();
        assert_eq!(args.get_default().unwrap(), "test0");
        assert_eq!(args.get("opt1").unwrap(), "test1");
        assert_eq!(args.get("opt2").unwrap(), "test2");
        assert_eq!(args.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn from_str_default2() {
        let argstr = "opt1=test1,test0";
        let args = ConnectorArgs::try_from(argstr).unwrap();
        assert_eq!(args.get_default(), None);
        assert_eq!(args.get("opt1").unwrap(), "test1");
    }

    #[test]
    pub fn builder() {
        let args = ConnectorArgs::new()
            .insert("arg1", "test1")
            .insert("arg2", "test2");
        assert_eq!(args.get("arg1").unwrap(), "test1");
        assert_eq!(args.get("arg2").unwrap(), "test2");
    }

    #[test]
    pub fn parse_empty() {
        let argstr = "opt1=test1,test0";
        let args = ConnectorArgs::try_from(argstr).unwrap();
        assert_eq!(args.get_default(), None);
    }
}
