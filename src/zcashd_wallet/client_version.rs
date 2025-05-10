use anyhow::Result;

use crate::{parse, parser::prelude::*};

#[derive(Clone, Copy)]
pub struct ClientVersion {
    version: u32,
    major: u32,
    minor: u32,
    revision: u32,
    build: u32,
}

impl ClientVersion {
    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn major(&self) -> u32 {
        self.major
    }

    pub fn minor(&self) -> u32 {
        self.minor
    }

    pub fn revision(&self) -> u32 {
        self.revision
    }

    pub fn build(&self) -> u32 {
        self.build
    }
}

impl PartialEq for ClientVersion {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

impl Eq for ClientVersion {}

impl PartialOrd for ClientVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.version.cmp(&other.version))
    }
}

impl std::hash::Hash for ClientVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.version.hash(state);
    }
}

impl ClientVersion {
    /// Parses a combined version integer into its components.
    ///
    /// Per zcashd's `clientversion.h`, the version is a 32-bit integer
    ///
    /// The integer is expected to be in the format:
    /// `major * 1_000_000 + minor * 10_000 + revision * 100 + build`
    ///
    /// # Arguments
    ///
    /// * `version` - The combined version integer.
    ///
    /// # Returns
    ///
    /// A `ClientVersion` struct with separated version components.
    pub fn from_integer(version: u32) -> Self {
        let major = version / 1_000_000;
        let remainder = version % 1_000_000;

        let minor = remainder / 10_000;
        let remainder = remainder % 10_000;

        let revision = remainder / 100;
        let build = remainder % 100;

        ClientVersion { version, major, minor, revision, build }
    }
}

impl Parse for ClientVersion {
    fn parse(p: &mut Parser) -> Result<Self> {
        let version: u32 = parse!(p, "ClientVersion")?;
        Ok(ClientVersion::from_integer(version))
    }
}

// Per zcashd's `clientversion.cpp`
impl std::fmt::Display for ClientVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.build < 25 {
            write!(
                f,
                "{}.{}.{}-beta{}",
                self.major,
                self.minor,
                self.revision,
                self.build + 1
            )
        } else if self.build < 50 {
            write!(
                f,
                "{}.{}.{}-rc{}",
                self.major,
                self.minor,
                self.revision,
                self.build - 24
            )
        } else if self.build == 50 {
            write!(f, "{}.{}.{}", self.major, self.minor, self.revision)
        } else {
            write!(
                f,
                "{}.{}.{}-{}",
                self.major,
                self.minor,
                self.revision,
                self.build - 50
            )
        }
    }
}

impl std::fmt::Debug for ClientVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
