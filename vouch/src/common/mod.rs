//! A module for data structures which are available to all super modules.
//!
//! This module contains data structures which are available to all super modules.
//! The number of data structures in this module should be minimized. The data structures
//! should be as simple as possible.
//!
//! Print statements are prohibited whithin this module. Logging is allowed.
use anyhow::Result;
use std::convert::TryFrom;

pub mod config;
pub mod fs;
pub mod index;

pub struct StoreTransaction<'a> {
    index_transaction: rusqlite::Transaction<'a>,
    git_transaction: crate::common::fs::GitTransaction,
}

impl<'a> StoreTransaction<'a> {
    pub fn new(index_transaction: rusqlite::Transaction<'a>) -> Result<Self> {
        Ok(Self {
            index_transaction,
            git_transaction: crate::common::fs::GitTransaction::new()?,
        })
    }

    pub fn index_tx(&self) -> &rusqlite::Transaction<'a> {
        &self.index_transaction
    }

    pub fn commit(mut self, message: &str) -> Result<()> {
        self.index_transaction.commit()?;
        self.git_transaction.commit(message)?;
        Ok(())
    }

    pub fn commit_index(self) -> Result<()> {
        self.index_transaction.commit()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct GitUrl(url::Url);

impl GitUrl {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn url(&self) -> &url::Url {
        &self.0
    }
}

impl std::convert::TryFrom<&str> for GitUrl {
    type Error = url::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = remove_suffix(value, ".git");
        Ok(Self {
            0: url::Url::parse(value)?,
        })
    }
}

impl std::convert::TryFrom<&String> for GitUrl {
    type Error = url::ParseError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let value = remove_suffix(value, ".git");
        Ok(Self {
            0: url::Url::parse(value)?,
        })
    }
}

fn remove_suffix<'a>(s: &'a str, p: &str) -> &'a str {
    if s.ends_with(p) {
        &s[..s.len() - p.len()]
    } else {
        s
    }
}

impl std::fmt::Display for GitUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl serde::Serialize for GitUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

pub struct UrlVisitor;

impl<'de> serde::de::Visitor<'de> for UrlVisitor {
    type Value = GitUrl;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a git URL which is parsable by url::Url")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(GitUrl::try_from(value)
            .map_err(|_| E::custom(format!("failed to parse URL \"{}\"", value)))?)
    }
}

impl<'de> serde::Deserialize<'de> for GitUrl {
    fn deserialize<D>(deserializer: D) -> Result<GitUrl, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(deserializer.deserialize_str(UrlVisitor)?)
    }
}

pub trait HashSansId {
    /// Compute hash without ID field.
    fn hash_sans_id<H: std::hash::Hasher>(&self, state: &mut H);
}
