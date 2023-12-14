use crate::version::Version;
use serde::Deserialize;
use url::Url;

mod lts_status {
    use serde::{Deserialize, Deserializer};

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(untagged)]
    enum LtsStatus {
        Nope(bool),
        Yes(String),
    }

    impl From<LtsStatus> for Option<String> {
        fn from(status: LtsStatus) -> Self {
            match status {
                LtsStatus::Nope(_) => None,
                LtsStatus::Yes(x) => Some(x),
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(LtsStatus::deserialize(deserializer)?.into())
    }
}

#[derive(Deserialize, Debug)]
pub struct IndexedNodeVersion {
    pub version: Version,
    #[serde(with = "lts_status")]
    pub lts: Option<String>,
}

/// Prints
///
/// ```rust
/// use crate::ni_remote::list;
/// ```
pub fn list(base_url: &Url) -> Result<Vec<IndexedNodeVersion>, crate::http::Error> {
    let index_json_url = format!("{base_url}/index.json");
    let resp = crate::http::get(&index_json_url)?;
    let mut value: Vec<IndexedNodeVersion> = resp.json()?;
    value.sort_by(|a, b| a.version.cmp(&b.version));
    Ok(value)
}
