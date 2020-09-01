/*!
Connector argument handler.
*/

use std::prelude::v1::*;

use crate::error::{Error, Result};

use core::convert::TryFrom;
use hashbrown::HashMap;

/// Argument wrapper for connectors
///
/// # Examples
///
/// Construct from a string:
/// ```
/// use memflow::connector::ConnectorArgs;
/// use std::convert::TryFrom;
///
/// let argstr = "opt1=test1,opt2=test2,opt3=test3";
/// let args = ConnectorArgs::parse(argstr).unwrap();
/// ```
///
/// Construct as builder:
/// ```
/// use memflow::connector::ConnectorArgs;
///
/// let args = ConnectorArgs::new()
///     .insert("arg1", "test1")
///     .insert("arg2", "test2");
/// ```
#[derive(Debug, Clone)]
pub struct ConnectorArgs {
    map: HashMap<String, String>,
}

impl ConnectorArgs {
    /// Creates an empty `ConnectorArgs` struct.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Creates a `ConnectorArgs` struct with a default (unnamed) value.
    pub fn with_default(value: &str) -> Self {
        Self::new().insert("default", value)
    }

    /// Tries to create a `ConnectorArgs` structure from an argument string.
    ///
    /// The argument string is a string of comma seperated key-value pairs.
    ///
    /// An argument string can just contain keys and values:
    /// `opt1=val1,opt2=val2,opt3=val3`
    ///
    /// The argument string can also contain a default value as the first entry
    /// which will be placed as a default argument:
    /// `default_value,opt1=val1,opt2=val2`
    ///
    /// This function can be used to initialize a connector from user input.
    pub fn parse(args: &str) -> Result<Self> {
        let mut map = HashMap::new();

        // if args != "" {
        let split = args.split(',');
        for (i, kv) in split.clone().enumerate() {
            let kvsplit = kv.split('=').collect::<Vec<_>>();
            if kvsplit.len() == 2 {
                map.insert(kvsplit[0].to_string(), kvsplit[1].to_string());
            } else if i == 0 && kv != "" {
                map.insert("default".to_string(), kv.to_string());
            }
        }
        // }

        Ok(Self { map })
    }

    /// Consumes self, inserts the given key-value pair and returns the self again.
    ///
    /// This function can be used as a builder pattern when programatically
    /// configuring connectors.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::connector::ConnectorArgs;
    ///
    /// let args = ConnectorArgs::new()
    ///     .insert("arg1", "test1")
    ///     .insert("arg2", "test2");
    /// ```
    pub fn insert(mut self, key: &str, value: &str) -> Self {
        self.map.insert(key.to_string(), value.to_string());
        self
    }

    /// Tries to retrieve an entry from the options map.
    /// If the entry was not found this function returns a `None` value.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }

    /// Tries to retrieve the default entry from the options map.
    /// If the entry was not found this function returns a `None` value.
    ///
    /// This function is a convenience wrapper for `args.get("default")`.
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
        ConnectorArgs::parse(args)
    }
}

impl TryFrom<String> for ConnectorArgs {
    type Error = Error;

    fn try_from(args: String) -> Result<Self> {
        ConnectorArgs::parse(&args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn from_str() {
        let argstr = "opt1=test1,opt2=test2,opt3=test3";
        let args = ConnectorArgs::parse(argstr).unwrap();
        assert_eq!(args.get("opt1").unwrap(), "test1");
        assert_eq!(args.get("opt2").unwrap(), "test2");
        assert_eq!(args.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn from_str_default() {
        let argstr = "test0,opt1=test1,opt2=test2,opt3=test3";
        let args = ConnectorArgs::parse(argstr).unwrap();
        assert_eq!(args.get_default().unwrap(), "test0");
        assert_eq!(args.get("opt1").unwrap(), "test1");
        assert_eq!(args.get("opt2").unwrap(), "test2");
        assert_eq!(args.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn from_str_default2() {
        let argstr = "opt1=test1,test0";
        let args = ConnectorArgs::parse(argstr).unwrap();
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
        let args = ConnectorArgs::parse(argstr).unwrap();
        assert_eq!(args.get_default(), None);
    }
}
