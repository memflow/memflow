/*!
Connector argument handler.
*/

use std::fmt;
use std::prelude::v1::*;

use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::types::size;

use core::convert::TryFrom;
use hashbrown::HashMap;

/// Argument wrapper for connectors
///
/// # Examples
///
/// Construct from a string:
/// ```
/// use memflow::plugins::Args;
/// use std::convert::TryFrom;
///
/// let argstr = "opt1=test1,opt2=test2,opt3=test3";
/// let args = Args::parse(argstr).unwrap();
/// ```
///
/// Construct as builder:
/// ```
/// use memflow::plugins::Args;
///
/// let args = Args::new()
///     .insert("arg1", "test1")
///     .insert("arg2", "test2");
/// ```
#[derive(Debug, Clone)]
pub struct Args {
    map: HashMap<String, String>,
}

impl fmt::Display for Args {
    /// Generates a string of key-value pairs containing the underlying data of the Args.
    ///
    /// This function will produce a string that can be properly parsed by the `parse` function again.
    ///
    /// # Remarks
    ///
    /// The sorting order of the underlying `HashMap` is random.
    /// This function only guarantees that the 'default' value (if it is set) will be the first element.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = Vec::new();

        if let Some(default) = self.get_default() {
            result.push(default.to_string());
        }

        result.extend(
            self.map
                .iter()
                .filter(|(key, _)| key.as_str() != "default")
                .map(|(key, value)| {
                    if value.contains(',') || value.contains('=') {
                        format!("{}=\"{}\"", key, value)
                    } else {
                        format!("{}={}", key, value)
                    }
                })
                .collect::<Vec<_>>(),
        );

        write!(f, "{}", result.join(","))
    }
}

impl Args {
    /// Creates an empty `Args` struct.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Creates a `Args` struct with a default (unnamed) value.
    pub fn with_default(value: &str) -> Self {
        Self::new().insert("default", value)
    }

    /// Tries to create a `Args` structure from an argument string.
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

        let quotes = args.split('"');
        let mut split = vec![];
        for (i, kv) in quotes.clone().enumerate() {
            if i % 2 == 0 {
                let s = kv.split(",");
                split.extend(s.map(|s| s.to_owned()));
            } else {
                if split.is_empty() {
                    split.push(kv.to_owned());
                } else {
                    let prev = split.pop().unwrap();
                    map.insert(prev[..prev.len() - 1].to_string(), kv.to_string());
                }
            }
        }

        for (i, kv) in split.iter().enumerate() {
            let kvsplit = kv.split('=').collect::<Vec<_>>();
            if kvsplit.len() == 2 {
                map.insert(kvsplit[0].to_string(), kvsplit[1].to_string());
            } else if i == 0 && !kv.is_empty() {
                map.insert("default".to_string(), kv.to_string());
            }
        }

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
    /// use memflow::plugins::Args;
    ///
    /// let args = Args::new()
    ///     .insert("arg1", "test1")
    ///     .insert("arg2", "test2");
    /// ```
    pub fn insert(mut self, key: &str, value: &str) -> Self {
        self.map.insert(key.to_string(), value.to_string());
        self
    }

    /// Tries to retrieve an entry from the options map.
    /// If the entry was not found this function returns a `None` value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| s.as_str())
    }

    /// Tries to retrieve the default entry from the options map.
    /// If the entry was not found this function returns a `None` value.
    ///
    /// This function is a convenience wrapper for `args.get("default")`.
    pub fn get_default(&self) -> Option<&str> {
        self.get("default")
    }
}

impl Default for Args {
    fn default() -> Self {
        Args::new()
    }
}

impl TryFrom<&str> for Args {
    type Error = Error;

    fn try_from(args: &str) -> Result<Self> {
        Args::parse(args)
    }
}

impl TryFrom<String> for Args {
    type Error = Error;

    fn try_from(args: String) -> Result<Self> {
        Args::parse(&args)
    }
}

impl From<Args> for String {
    fn from(args: Args) -> Self {
        args.to_string()
    }
}

/// Validator for connector arguments
///
/// # Examples
///
/// Builder:
/// ```
/// use memflow::plugins::{ArgsValidator, ArgDescriptor};
///
/// let validator = ArgsValidator::new()
///     .arg(ArgDescriptor::new("default"))
///     .arg(ArgDescriptor::new("arg1"));
/// ```
#[derive(Debug)]
pub struct ArgsValidator {
    args: Vec<ArgDescriptor>,
}

impl Default for ArgsValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ArgsValidator {
    /// Creates an empty `ArgsValidator` struct.
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    /// Adds an `ArgDescriptor` to the validator and returns itself.
    pub fn arg(mut self, arg: ArgDescriptor) -> Self {
        self.args.push(arg);
        self
    }

    pub fn validate(&self, args: &Args) -> Result<()> {
        // check if all given args exist
        for arg in args.map.iter() {
            if !self.args.iter().any(|a| a.name == *arg.0) {
                return Err(Error(ErrorOrigin::ArgsValidator, ErrorKind::ArgNotExists)
                    .log_error(format!("argument {} does not exist", arg.0)));
            }
        }

        for arg in self.args.iter() {
            // check if required args are set
            if arg.required && args.get(&arg.name).is_none() {
                return Err(
                    Error(ErrorOrigin::ArgsValidator, ErrorKind::RequiredArgNotFound).log_error(
                        format!("argument {} is required but could not be found", arg.name),
                    ),
                );
            }

            // check if validate matches
            if let Some(validator) = &arg.validator {
                if let Some(value) = args.get(&arg.name) {
                    if let Err(err) = validator(value) {
                        return Err(Error(ErrorOrigin::ArgsValidator, ErrorKind::ArgValidation)
                            .log_error(format!("argument {} is invalid: {}", arg.name, err)));
                    }
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for ArgsValidator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (idx, arg) in self.args.iter().enumerate() {
            if idx < self.args.len() - 1 {
                writeln!(f, "{}", arg).ok();
            } else {
                write!(f, "{}", arg).ok();
            }
        }
        Ok(())
    }
}

pub type ArgValidator = Box<dyn Fn(&str) -> ::std::result::Result<(), &'static str>>;

/// Describes a single validator argument.
///
/// # Examples
///
/// Builder:
/// ```
/// use memflow::plugins::ArgDescriptor;
///
/// let desc = ArgDescriptor::new("default")
///     .description("default argument description")
///     .required(true);
/// ```
pub struct ArgDescriptor {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub validator: Option<ArgValidator>,
}

impl ArgDescriptor {
    /// Creates a new `ArgDescriptor` with the given argument name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: None,
            required: false,
            validator: None,
        }
    }

    /// Set the description for this argument.
    ///
    /// By default the description is `None`.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_owned());
        self
    }

    /// Set the required state for this argument.
    ///
    /// By default arguments are optional.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets the validator function for this argument.
    ///
    /// By default no validator is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::plugins::ArgDescriptor;
    ///
    /// let desc = ArgDescriptor::new("default").validator(Box::new(|arg| {
    ///     match arg == "valid_option" {
    ///         true => Ok(()),
    ///         false => Err("argument must be 'valid_option'"),
    ///     }
    /// }));
    /// ```
    pub fn validator(mut self, validator: ArgValidator) -> Self {
        self.validator = Some(validator);
        self
    }
}

impl fmt::Display for ArgDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}{}",
            self.name,
            self.description
                .as_ref()
                .unwrap_or(&"no description available".to_owned()),
            if self.required { " (required)" } else { "" },
        )
    }
}

impl fmt::Debug for ArgDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}{}",
            self.name,
            self.description
                .as_ref()
                .unwrap_or(&"no description available".to_owned()),
            if self.required { " (required)" } else { "" },
        )
    }
}

pub fn parse_pagecache(args: &Args) -> Result<Option<(usize, u64)>> {
    match args.get("pagecache").unwrap_or("default") {
        "default" => Ok(Some((0, 0))),
        "none" => Ok(None),
        size => Ok(Some(parse_pagecache_args(size)?)),
    }
}

fn parse_pagecache_args(vargs: &str) -> Result<(usize, u64)> {
    let mut sp = vargs.splitn(2, ';');
    let (size, time) = (
        sp.next().ok_or_else(|| {
            Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                .log_error("Failed to parse Page Cache size")
        })?,
        sp.next().unwrap_or("0"),
    );

    let (size, size_mul) = {
        let mul_arr = &[
            (size::kb(1), ["kb", "k"]),
            (size::mb(1), ["mb", "m"]),
            (size::gb(1), ["gb", "g"]),
        ];

        mul_arr
            .iter()
            .flat_map(|(m, e)| e.iter().map(move |e| (*m, e)))
            .find_map(|(m, e)| {
                if size.to_lowercase().ends_with(e) {
                    Some((size.trim_end_matches(e), m))
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                    .log_error("Invalid Page Cache size unit (or none)!")
            })?
    };

    let size = usize::from_str_radix(size, 16).map_err(|_| {
        Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
            .log_error("Failed to parse Page Cache size")
    })?;

    let size = size * size_mul;

    let time = time.parse::<u64>().map_err(|_| {
        Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
            .log_error("Failed to parse Page Cache validity time")
    })?;

    Ok((size, time))
}

pub fn parse_vatcache(args: &Args) -> Result<Option<(usize, u64)>> {
    match args.get("vatcache").unwrap_or("default") {
        "default" => Ok(Some((0, 0))),
        "none" => Ok(None),
        size => Ok(Some(parse_vatcache_args(size)?)),
    }
}

fn parse_vatcache_args(vargs: &str) -> Result<(usize, u64)> {
    let mut sp = vargs.splitn(2, ';');
    let (size, time) = (
        sp.next().ok_or_else(|| {
            Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                .log_error("Failed to parse VAT size")
        })?,
        sp.next().unwrap_or("0"),
    );
    let size = usize::from_str_radix(size, 16).map_err(|_| {
        Error(ErrorOrigin::OsLayer, ErrorKind::Configuration).log_error("Failed to parse VAT size")
    })?;
    let time = time.parse::<u64>().map_err(|_| {
        Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
            .log_error("Failed to parse VAT validity time")
    })?;
    Ok((size, time))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn from_str() {
        let argstr = "opt1=test1,opt2=test2,opt3=test3";
        let args = Args::parse(argstr).unwrap();
        assert_eq!(args.get("opt1").unwrap(), "test1");
        assert_eq!(args.get("opt2").unwrap(), "test2");
        assert_eq!(args.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn from_str_default() {
        let argstr = "test0,opt1=test1,opt2=test2,opt3=test3";
        let args = Args::parse(argstr).unwrap();
        assert_eq!(args.get_default().unwrap(), "test0");
        assert_eq!(args.get("opt1").unwrap(), "test1");
        assert_eq!(args.get("opt2").unwrap(), "test2");
        assert_eq!(args.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn from_str_default2() {
        let argstr = "opt1=test1,test0";
        let args = Args::parse(argstr).unwrap();
        assert_eq!(args.get_default(), None);
        assert_eq!(args.get("opt1").unwrap(), "test1");
    }

    #[test]
    pub fn builder() {
        let args = Args::new().insert("arg1", "test1").insert("arg2", "test2");
        assert_eq!(args.get("arg1").unwrap(), "test1");
        assert_eq!(args.get("arg2").unwrap(), "test2");
    }

    #[test]
    pub fn parse_empty() {
        let argstr = "opt1=test1,test0";
        let args = Args::parse(argstr).unwrap();
        assert_eq!(args.get_default(), None);
    }

    #[test]
    pub fn to_string() {
        let argstr = "opt1=test1,opt2=test2,opt3=test3";
        let args = Args::parse(argstr).unwrap();
        let args2 = Args::parse(&args.to_string()).unwrap();
        assert_eq!(args2.get_default(), None);
        assert_eq!(args2.get("opt1").unwrap(), "test1");
        assert_eq!(args2.get("opt2").unwrap(), "test2");
        assert_eq!(args2.get("opt3").unwrap(), "test3");
    }

    // TODO: test non default first to string
    #[test]
    pub fn to_string_with_default() {
        let argstr = "test0,opt1=test1,opt2=test2,opt3=test3";
        let args = Args::parse(argstr).unwrap();
        let args2 = Args::parse(&args.to_string()).unwrap();
        assert_eq!(args2.get_default().unwrap(), "test0");
        assert_eq!(args2.get("opt1").unwrap(), "test1");
        assert_eq!(args2.get("opt2").unwrap(), "test2");
        assert_eq!(args2.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn double_quotes() {
        let argstr = "opt1=test1,test0,opt2=\"test2,test3\"";
        let args = Args::parse(argstr).unwrap();
        let args2 = Args::parse(&args.to_string()).unwrap();
        assert_eq!(args2.get_default(), None);
        assert_eq!(args2.get("opt1").unwrap(), "test1");
        assert_eq!(args2.get("opt2").unwrap(), "test2,test3");
    }

    #[test]
    pub fn double_quotes_eq() {
        let argstr = "opt1=test1,test0,opt2=\"test2,test3=test4\"";
        let args = Args::parse(argstr).unwrap();
        let args2 = Args::parse(&args.to_string()).unwrap();
        assert_eq!(args2.get_default(), None);
        assert_eq!(args2.get("opt1").unwrap(), "test1");
        assert_eq!(args2.get("opt2").unwrap(), "test2,test3=test4");
    }

    #[test]
    pub fn validator_success() {
        let validator = ArgsValidator::new()
            .arg(ArgDescriptor::new("default"))
            .arg(ArgDescriptor::new("opt1"));

        let argstr = "test0,opt1=test1";
        let args = Args::parse(argstr).unwrap();

        assert_eq!(validator.validate(&args), Ok(()));
    }

    #[test]
    pub fn validator_success_optional() {
        let validator = ArgsValidator::new()
            .arg(ArgDescriptor::new("default").required(true))
            .arg(ArgDescriptor::new("opt1").required(false));

        let argstr = "test0";
        let args = Args::parse(argstr).unwrap();

        assert_eq!(validator.validate(&args), Ok(()));
    }

    #[test]
    pub fn validator_error_required() {
        let validator = ArgsValidator::new()
            .arg(ArgDescriptor::new("default").required(true))
            .arg(ArgDescriptor::new("opt1").required(true));

        let argstr = "test0";
        let args = Args::parse(argstr).unwrap();

        assert_eq!(
            validator.validate(&args),
            Err(Error(
                ErrorOrigin::ArgsValidator,
                ErrorKind::RequiredArgNotFound
            ))
        );
    }

    #[test]
    pub fn validator_error_notexist() {
        let validator = ArgsValidator::new()
            .arg(ArgDescriptor::new("default"))
            .arg(ArgDescriptor::new("opt1"));

        let argstr = "test0,opt2=arg2";
        let args = Args::parse(argstr).unwrap();

        assert_eq!(
            validator.validate(&args),
            Err(Error(ErrorOrigin::ArgsValidator, ErrorKind::ArgNotExists))
        );
    }

    #[test]
    pub fn validator_validate_success() {
        let validator =
            ArgsValidator::new().arg(ArgDescriptor::new("default").validator(Box::new(|arg| {
                match arg == "valid_option" {
                    true => Ok(()),
                    false => Err("argument must be 'valid_option'"),
                }
            })));

        let argstr = "valid_option";
        let args = Args::parse(argstr).unwrap();

        assert_eq!(validator.validate(&args), Ok(()));
    }

    #[test]
    pub fn validator_validate_fail() {
        let validator =
            ArgsValidator::new().arg(ArgDescriptor::new("default").validator(Box::new(|arg| {
                match arg == "valid_option" {
                    true => Ok(()),
                    false => Err("argument must be 'valid_option'"),
                }
            })));

        let argstr = "invalid_option";
        let args = Args::parse(argstr).unwrap();

        assert_eq!(
            validator.validate(&args),
            Err(Error(ErrorOrigin::ArgsValidator, ErrorKind::ArgValidation))
        );
    }
}
