/*!
Connector argument handler.
*/

use std::fmt;
use std::prelude::v1::*;

use crate::error::{Error, ErrorKind, ErrorOrigin, Result};

use cglue::{repr_cstring::ReprCString, vec::CVec};

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
/// let args: Args = argstr.parse().unwrap();
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
#[repr(C)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Args {
    // Just how many args do you have usually?
    // Hashmap performance improvements may not be worth the complexity
    // C/C++ users would have in constructing arguments structure.
    args: CVec<ArgEntry>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ArgEntry {
    key: ReprCString,
    value: ReprCString,
}

impl<T: Into<ReprCString>> From<(T, T)> for ArgEntry {
    fn from((key, value): (T, T)) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
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
            self.args
                .iter()
                .filter(|e| &*e.key != "default")
                .map(|ArgEntry { key, value }| {
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

impl std::str::FromStr for Args {
    type Err = crate::error::Error;

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
    fn from_str(s: &str) -> Result<Self> {
        let split = split_str_args(s, ',').collect::<Vec<_>>();

        let mut map = HashMap::new();
        for (i, kv) in split.iter().enumerate() {
            let kvsplit = split_str_args(kv, '=').collect::<Vec<_>>();
            if kvsplit.len() == 2 {
                map.insert(kvsplit[0].to_string(), kvsplit[1].to_string());
            } else if i == 0 && !kv.is_empty() {
                map.insert("default".to_string(), kv.to_string());
            }
        }

        Ok(Self {
            args: map.into_iter().map(<_>::into).collect::<Vec<_>>().into(),
        })
    }
}

impl Default for Args {
    /// Creates an empty `Args` struct.
    fn default() -> Self {
        Self {
            args: Default::default(),
        }
    }
}

impl Args {
    /// Creates an empty `Args` struct.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a `Args` struct with a default (unnamed) value.
    pub fn with_default(value: &str) -> Self {
        Self::new().insert("default", value)
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
        if let Some(a) = self.args.iter_mut().find(|a| &*a.key == key) {
            a.value = value.into();
        } else {
            self.args.push((key, value).into());
        }
        self
    }

    /// Tries to retrieve an entry from the options map.
    /// If the entry was not found this function returns a `None` value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.args
            .iter()
            .filter(|a| &*a.key == key)
            .map(|a| &*a.value)
            .next()
    }

    /// Tries to retrieve the default entry from the options map.
    /// If the entry was not found this function returns a `None` value.
    ///
    /// This function is a convenience wrapper for `args.get("default")`.
    pub fn get_default(&self) -> Option<&str> {
        self.get("default")
    }
}

impl TryFrom<&str> for Args {
    type Error = Error;

    fn try_from(args: &str) -> Result<Self> {
        args.parse()
    }
}

impl TryFrom<String> for Args {
    type Error = Error;

    fn try_from(args: String) -> Result<Self> {
        args.parse()
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
        for arg in args.args.iter() {
            if !self.args.iter().any(|a| a.name == *arg.key) {
                return Err(Error(ErrorOrigin::ArgsValidator, ErrorKind::ArgNotExists)
                    .log_error(format!("argument {} does not exist", &*arg.key)));
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
/// let desc = ArgDescriptor::new("cache_size")
///     .description("cache_size argument description")
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
    /// let desc = ArgDescriptor::new("cache_size").validator(Box::new(|arg| {
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

/// Split a string into a list of separate parts based on ':' delimiter
///
/// This is a more advanced version of splitting that allows to do some basic escaping with
/// quotation marks.
///
/// # Examples
///
/// ```
/// use memflow::plugins::args::split_str_args;
///
/// let v: Vec<_> = split_str_args("a:b:c", ':').collect();
/// assert_eq!(v, ["a", "b", "c"]);
///
/// let v: Vec<_> = split_str_args("a::c", ':').collect();
/// assert_eq!(v, ["a", "", "c"]);
///
/// let v: Vec<_> = split_str_args("a:\"hello\":c", ':').collect();
/// assert_eq!(v, ["a", "hello", "c"]);
///
/// let v: Vec<_> = split_str_args("a:\"hel:lo\":c", ':').collect();
/// assert_eq!(v, ["a", "hel:lo", "c"]);
///
/// let v: Vec<_> = split_str_args("a:\"hel:lo:c", ':').collect();
/// assert_eq!(v, ["a", "\"hel:lo:c"]);
///
/// let v: Vec<_> = split_str_args("a:'hel\":lo\"':c", ':').collect();
/// assert_eq!(v, ["a", "hel\":lo\"", "c"]);
///
/// let v: Vec<_> = split_str_args("a:hel\":lo\":c", ':').collect();
/// assert_eq!(v, ["a", "hel\":lo\"", "c"]);
/// ```
pub fn split_str_args(inp: &str, split_char: char) -> impl Iterator<Item = &str> {
    let mut prev_char = '\0';
    let mut quotation_char = None;

    const VALID_QUOTES: &str = "\"'`";
    assert!(!VALID_QUOTES.contains(split_char));

    inp.split(move |c| {
        let mut ret = false;

        // found an unescaped quote
        if VALID_QUOTES.contains(c) && prev_char != '\\' {
            // scan string up until we find the same quotation char again
            match quotation_char {
                Some(qc) if qc == c => {
                    quotation_char = None;
                }
                None => quotation_char = Some(c),
                _ => (),
            }
        }

        if quotation_char.is_none() && c == split_char {
            ret = true;
        }

        prev_char = c;
        ret
    })
    .map(|s| {
        if let Some(c) = s.chars().next().and_then(|a| {
            if s.ends_with(a) && VALID_QUOTES.contains(a) {
                Some(a)
            } else {
                None
            }
        }) {
            s.split_once(c)
                .and_then(|(_, a)| a.rsplit_once(c))
                .map(|(a, _)| a)
                .unwrap_or("")
        } else {
            s
        }
    })
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
        let args: Args = argstr.parse().unwrap();
        assert_eq!(args.get("opt1").unwrap(), "test1");
        assert_eq!(args.get("opt2").unwrap(), "test2");
        assert_eq!(args.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn from_str_default() {
        let argstr = "test0,opt1=test1,opt2=test2,opt3=test3";
        let args: Args = argstr.parse().unwrap();
        assert_eq!(args.get_default().unwrap(), "test0");
        assert_eq!(args.get("opt1").unwrap(), "test1");
        assert_eq!(args.get("opt2").unwrap(), "test2");
        assert_eq!(args.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn from_str_default2() {
        let argstr = "opt1=test1,test0";
        let args: Args = argstr.parse().unwrap();
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
        let _: Args = argstr.parse().unwrap();
    }

    #[test]
    pub fn to_string() {
        let argstr = "opt1=test1,opt2=test2,opt3=test3";
        let args: Args = argstr.parse().unwrap();
        let args2: Args = args.to_string().parse().unwrap();
        assert_eq!(args2.get_default(), None);
        assert_eq!(args2.get("opt1").unwrap(), "test1");
        assert_eq!(args2.get("opt2").unwrap(), "test2");
        assert_eq!(args2.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn to_string_with_default() {
        let argstr = "test0,opt1=test1,opt2=test2,opt3=test3";
        let args: Args = argstr.parse().unwrap();
        let args2: Args = args.to_string().parse().unwrap();
        assert_eq!(args2.get_default().unwrap(), "test0");
        assert_eq!(args2.get("opt1").unwrap(), "test1");
        assert_eq!(args2.get("opt2").unwrap(), "test2");
        assert_eq!(args2.get("opt3").unwrap(), "test3");
    }

    #[test]
    pub fn double_quotes() {
        let argstr = "opt1=test1,test0,opt2=\"test2,test3\"";
        let args: Args = argstr.parse().unwrap();
        let args2: Args = args.to_string().parse().unwrap();
        assert_eq!(args2.get("opt1").unwrap(), "test1");
        assert_eq!(args2.get("opt2").unwrap(), "test2,test3");
    }

    #[test]
    pub fn double_quotes_eq() {
        let argstr = "opt1=test1,test0,opt2=\"test2,test3=test4\"";
        let args: Args = argstr.parse().unwrap();
        let args2: Args = args.to_string().parse().unwrap();
        assert_eq!(args2.get("opt1").unwrap(), "test1");
        assert_eq!(args2.get("opt2").unwrap(), "test2,test3=test4");
    }

    #[test]
    pub fn slashes() {
        let argstr = "device=vmware://,remote=rpc://insecure:computername.local";
        let args: Args = argstr.parse().unwrap();
        let args2: Args = args.to_string().parse().unwrap();
        assert_eq!(args2.get("device").unwrap(), "vmware://");
        assert_eq!(
            args2.get("remote").unwrap(),
            "rpc://insecure:computername.local"
        );
    }

    #[test]
    pub fn slashes_quotes_split() {
        let v: Vec<_> = split_str_args(
            "url1=\"uri://ip=test:test@test,test\",url2=\"test:test@test.de,test2:test2@test2.de\"",
            ',',
        )
        .collect();
        assert_eq!(
            v,
            [
                "url1=\"uri://ip=test:test@test,test\"",
                "url2=\"test:test@test.de,test2:test2@test2.de\""
            ]
        );
    }

    #[test]
    pub fn slashes_quotes() {
        let argstr = "device=\"RAWUDP://ip=127.0.0.1\"";
        let args: Args = argstr.parse().unwrap();
        let args2: Args = args.to_string().parse().unwrap();
        assert_eq!(args2.get("device").unwrap(), "RAWUDP://ip=127.0.0.1");
    }

    #[test]
    pub fn slashes_mixed_quotes() {
        let argstr = "device=`RAWUDP://ip=127.0.0.1`";
        let args: Args = argstr.parse().unwrap();
        assert_eq!(args.get("device").unwrap(), "RAWUDP://ip=127.0.0.1");

        let arg2str = args.to_string();
        assert_eq!(arg2str, "device=\"RAWUDP://ip=127.0.0.1\"");

        let args2: Args = arg2str.parse().unwrap();
        assert_eq!(args2.get("device").unwrap(), "RAWUDP://ip=127.0.0.1");
    }

    #[test]
    pub fn slashes_quotes_complex() {
        let argstr =
            "url1=\"uri://ip=test:test@test,test\",url2=\"test:test@test.de,test2:test2@test2.de\"";
        let args: Args = argstr.parse().unwrap();
        let args2: Args = args.to_string().parse().unwrap();
        assert_eq!(args2.get("url1").unwrap(), "uri://ip=test:test@test,test");
        assert_eq!(
            args2.get("url2").unwrap(),
            "test:test@test.de,test2:test2@test2.de"
        );
    }

    #[test]
    pub fn validator_success() {
        let validator = ArgsValidator::new()
            .arg(ArgDescriptor::new("default"))
            .arg(ArgDescriptor::new("opt1"));

        let argstr = "test0,opt1=test1";
        let args: Args = argstr.parse().unwrap();

        assert_eq!(validator.validate(&args), Ok(()));
    }

    #[test]
    pub fn validator_success_optional() {
        let validator = ArgsValidator::new().arg(ArgDescriptor::new("opt1").required(false));

        let args: Args = "".parse().unwrap();

        assert_eq!(validator.validate(&args), Ok(()));
    }

    #[test]
    pub fn validator_error_required() {
        let validator = ArgsValidator::new().arg(ArgDescriptor::new("opt1").required(true));

        let args: Args = "".parse().unwrap();

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
        let validator = ArgsValidator::new().arg(ArgDescriptor::new("opt1"));

        let argstr = "opt2=arg2";
        let args: Args = argstr.parse().unwrap();

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

        let argstr = "default=valid_option";
        let args: Args = argstr.parse().unwrap();

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
        let args: Args = argstr.parse().unwrap();

        assert_eq!(
            validator.validate(&args),
            Err(Error(ErrorOrigin::ArgsValidator, ErrorKind::ArgValidation))
        );
    }
}
