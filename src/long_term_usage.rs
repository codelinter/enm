use crate::ni_remote::IndexedNodeVersion;
use std::fmt::Display;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum LongTermType {
    /// lts-*, lts/*
    Latest,
    /// lts-erbium, lts/erbium
    CodeName(String),
}

impl From<&str> for LongTermType {
    fn from(s: &str) -> Self {
        if s == "*" || s == "latest" {
            Self::Latest
        } else {
            Self::CodeName(s.to_string())
        }
    }
}

impl Display for LongTermType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Latest => f.write_str("latest"),
            Self::CodeName(s) => f.write_str(s),
        }
    }
}

impl LongTermType {
    pub fn pick_latest<'vec>(
        &self,
        versions: &'vec [IndexedNodeVersion],
    ) -> Option<&'vec IndexedNodeVersion> {
        match self {
            Self::Latest => versions.iter().filter(|x| x.lts.is_some()).last(),
            Self::CodeName(s) => versions
                .iter()
                .filter(|x| match &x.lts {
                    None => false,
                    Some(x) => s.to_lowercase() == x.to_lowercase(),
                })
                .last(),
        }
    }
}
