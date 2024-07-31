use core::convert::{TryFrom, TryInto};
use std::prelude::v1::*;

use super::{ConnectorArgs, ConnectorInstanceArcBox, Inventory, OsArgs, OsInstanceArcBox};
use crate::error::{Error, ErrorKind, ErrorOrigin, Result};

pub enum BuildStep<'a> {
    Connector {
        name: &'a str,
        args: Option<ConnectorArgs>,
    },
    Os {
        name: &'a str,
        args: Option<OsArgs>,
    },
}

impl<'a> BuildStep<'a> {
    /// Parse input string and construct steps for building a connector.
    ///
    /// Name and arguments are separated by `:`, for example:
    ///
    /// `kvm:5`, or `qemu:win10:memmap=map`.
    pub fn new_connector(input: &'a str) -> Result<Self> {
        let (name, args) = input.split_once(':').unwrap_or((input, ""));

        Ok(Self::Connector {
            name,
            args: if args.is_empty() {
                None
            } else {
                Some(str::parse(args)?)
            },
        })
    }

    /// Parse input string and construct steps for building an OS.
    ///
    /// Name and arguments are separated by `:`, for example:
    ///
    /// `win32`, or `win32::dtb=0xdeadbeef`.
    pub fn new_os(input: &'a str) -> Result<Self> {
        let (name, args) = input.split_once(':').unwrap_or((input, ""));

        Ok(Self::Os {
            name,
            args: if args.is_empty() {
                None
            } else {
                Some(str::parse(args)?)
            },
        })
    }

    /// Validate whether the next build step is compatible with the current one.
    pub fn validate_next(&self, next: &Self) -> bool {
        !matches!(
            (self, next),
            (BuildStep::Connector { .. }, BuildStep::Connector { .. })
                | (BuildStep::Os { .. }, BuildStep::Os { .. })
        )
    }
}

fn builder_from_args<'a>(
    connectors: impl Iterator<Item = (usize, &'a str)>,
    os_layers: impl Iterator<Item = (usize, &'a str)>,
) -> Result<Vec<BuildStep<'a>>> {
    let mut layers = connectors
        .map(|(i, a)| BuildStep::new_connector(a).map(|a| (i, a)))
        .chain(os_layers.map(|(i, a)| BuildStep::new_os(a).map(|a| (i, a))))
        .collect::<Result<Vec<_>>>()?;

    layers.sort_by(|(a, _), (b, _)| a.cmp(b));

    if layers.windows(2).any(|w| !w[0].1.validate_next(&w[1].1)) {
        return Err(
            Error(ErrorOrigin::Other, ErrorKind::ArgValidation).log_error(
                "invalid builder configuration, build steps cannot be used in the given order",
            ),
        );
    }

    Ok(layers.into_iter().map(|(_, s)| s).collect())
}

/// Precompiled connector chain.
///
/// Use this with [`Inventory::builder`](Inventory::builder).
pub struct ConnectorChain<'a>(Vec<BuildStep<'a>>);

impl<'a> ConnectorChain<'a> {
    /// Build a new connector chain.
    ///
    /// Arguments are iterators of command line arguments with their position and value. The
    /// position will be used to sort them and validate whether they are in correct order.
    pub fn new(
        connectors: impl Iterator<Item = (usize, &'a str)>,
        os_layers: impl Iterator<Item = (usize, &'a str)>,
    ) -> Result<Self> {
        let steps = builder_from_args(connectors, os_layers)?;
        steps.try_into()
    }
}

impl<'a> TryFrom<Vec<BuildStep<'a>>> for ConnectorChain<'a> {
    type Error = Error;

    fn try_from(steps: Vec<BuildStep<'a>>) -> Result<Self> {
        if !matches!(steps.last(), Some(BuildStep::Connector { .. })) {
            return Err(
                Error(ErrorOrigin::Other, ErrorKind::ArgValidation).log_error(
                    "invalid builder configuration, last build step has to be a connector",
                ),
            );
        }

        Ok(Self(steps))
    }
}

/// Precompiled os chain.
///
/// Use this with [`Inventory::builder`](Inventory::builder).
pub struct OsChain<'a>(Vec<BuildStep<'a>>);

impl<'a> OsChain<'a> {
    /// Build a new OS chain.
    ///
    /// Arguments are iterators of command line arguments with their position and value. The
    /// position will be used to sort them and validate whether they are in correct order.
    pub fn new(
        connectors: impl Iterator<Item = (usize, &'a str)>,
        os_layers: impl Iterator<Item = (usize, &'a str)>,
    ) -> Result<Self> {
        let steps = builder_from_args(connectors, os_layers)?;
        steps.try_into()
    }
}

impl<'a> TryFrom<Vec<BuildStep<'a>>> for OsChain<'a> {
    type Error = Error;

    fn try_from(steps: Vec<BuildStep<'a>>) -> Result<Self> {
        if !matches!(steps.last(), Some(BuildStep::Os { .. })) {
            return Err(Error(ErrorOrigin::Other, ErrorKind::ArgValidation)
                .log_error("invalid builder configuration, last build step has to be a os"));
        }

        Ok(Self(steps))
    }
}

/// BuilderEmpty is the starting builder that allows to either call `connector`, or `os`.
pub struct BuilderEmpty<'a> {
    inventory: &'a mut Inventory,
}

impl<'a> BuilderEmpty<'a> {
    pub fn new(inventory: &'a mut Inventory) -> Self {
        Self { inventory }
    }

    /// Adds a Connector instance to the build chain
    ///
    /// # Arguments
    ///
    /// * `name` - name of the connector
    pub fn connector(self, name: &'a str) -> OsBuilder<'a> {
        OsBuilder {
            inventory: self.inventory,
            steps: vec![BuildStep::Connector { name, args: None }],
        }
    }

    /// Adds an OS instance to the build chain
    ///
    /// # Arguments
    ///
    /// * `name` - name of the target OS
    pub fn os(self, name: &'a str) -> ConnectorBuilder<'a> {
        ConnectorBuilder {
            inventory: self.inventory,
            steps: vec![BuildStep::Os { name, args: None }],
        }
    }

    /// Chains multiple pre-validated steps, resulting in an Os ready-to-build.
    ///
    /// # Arguments
    ///
    /// * `chain` - steps to initialize the builder with.
    pub fn os_chain(self, chain: OsChain<'a>) -> ConnectorBuilder<'a> {
        ConnectorBuilder {
            inventory: self.inventory,
            steps: chain.0,
        }
    }

    /// Chains multiple pre-validated steps, resulting in a connector ready-to-build.
    ///
    /// # Arguments
    ///
    /// * `chain` - steps to initialize the builder with.
    pub fn connector_chain(self, chain: ConnectorChain<'a>) -> OsBuilder<'a> {
        OsBuilder {
            inventory: self.inventory,
            steps: chain.0,
        }
    }
}

/// ConnectorBuilder creates a new connector instance with the previous os step as an input.
pub struct ConnectorBuilder<'a> {
    inventory: &'a mut Inventory,
    steps: Vec<BuildStep<'a>>,
}

impl<'a> ConnectorBuilder<'a> {
    /// Adds a Connector instance to the build chain
    ///
    /// # Arguments
    ///
    /// * `name` - name of the connector
    pub fn connector(self, name: &'a str) -> OsBuilder<'a> {
        let mut steps = self.steps;
        steps.push(BuildStep::Connector { name, args: None });
        OsBuilder {
            inventory: self.inventory,
            steps,
        }
    }

    /// Appends arguments to the previously added OS.
    ///
    /// # Arguments
    ///
    /// * `os_args` - the arguments to be passed to the previously added OS
    pub fn args(mut self, os_args: OsArgs) -> ConnectorBuilder<'a> {
        if let Some(BuildStep::Os { name: _, args }) = self.steps.iter_mut().last() {
            *args = Some(os_args);
        }
        self
    }

    /// Builds the final chain of Connectors and OS and returns the last OS.
    ///
    /// Each created connector / os instance is fed into the next os / connector instance as an argument.
    /// If any build step fails the function returns an error.
    pub fn build(self) -> Result<OsInstanceArcBox<'static>> {
        let mut connector: Option<ConnectorInstanceArcBox<'static>> = None;
        let mut os: Option<OsInstanceArcBox<'static>> = None;
        for step in self.steps.iter() {
            match step {
                BuildStep::Connector { name, args } => {
                    connector = Some(self.inventory.instantiate_connector(
                        name,
                        os,
                        args.as_ref(),
                    )?);
                    os = None;
                }
                BuildStep::Os { name, args } => {
                    os = Some(
                        self.inventory
                            .instantiate_os(name, connector, args.as_ref())?,
                    );
                    connector = None;
                }
            };
        }
        os.ok_or(Error(ErrorOrigin::Inventory, ErrorKind::Configuration))
    }
}

/// OsBuilder creates a new os instance with the previous connector step as an input
pub struct OsBuilder<'a> {
    inventory: &'a mut Inventory,
    steps: Vec<BuildStep<'a>>,
}

impl<'a> OsBuilder<'a> {
    /// Adds an OS instance to the build chain
    ///
    /// # Arguments
    ///
    /// * `name` - name of the target OS
    pub fn os(self, name: &'a str) -> ConnectorBuilder<'a> {
        let mut steps = self.steps;
        steps.push(BuildStep::Os { name, args: None });
        ConnectorBuilder {
            inventory: self.inventory,
            steps,
        }
    }

    /// Appends arguments to the previously added Connector.
    ///
    /// # Arguments
    ///
    /// * `conn_args` - the arguments to be passed to the previously added Connector
    pub fn args(mut self, conn_args: ConnectorArgs) -> OsBuilder<'a> {
        if let Some(BuildStep::Connector { name: _, args }) = self.steps.iter_mut().last() {
            *args = Some(conn_args);
        }
        self
    }

    /// Builds the final chain of Connectors and OS and returns the last Connector.
    ///
    /// Each created connector / os instance is fed into the next os / connector instance as an argument.
    /// If any build step fails the function returns an error.
    pub fn build(self) -> Result<ConnectorInstanceArcBox<'static>> {
        let mut connector: Option<ConnectorInstanceArcBox<'static>> = None;
        let mut os: Option<OsInstanceArcBox<'static>> = None;
        for step in self.steps.iter() {
            match step {
                BuildStep::Connector { name, args } => {
                    connector = Some(self.inventory.instantiate_connector(
                        name,
                        os,
                        args.as_ref(),
                    )?);
                    os = None;
                }
                BuildStep::Os { name, args } => {
                    os = Some(
                        self.inventory
                            .instantiate_os(name, connector, args.as_ref())?,
                    );
                    connector = None;
                }
            };
        }
        connector.ok_or(Error(ErrorOrigin::Inventory, ErrorKind::Configuration))
    }
}
