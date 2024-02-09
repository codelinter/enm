use crate::version::Version;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum UserVersion {
    OnlyMajor(u64),
    MajorMinor(u64, u64),
    Full(Version),
}

impl UserVersion {
    pub fn to_version<'a, T>(
        &self,
        available_versions: T,
        config: &crate::app_config::AppConfig,
    ) -> Option<&'a Version>
    where
        T: IntoIterator<Item = &'a Version>,
    {
        available_versions
            .into_iter()
            .filter(|x| self.matches(x, config))
            .max()
    }

    pub fn alias_name(&self) -> Option<String> {
        match self {
            Self::Full(version) => version.alias_name(),
            _ => None,
        }
    }

    pub fn matches(&self, version: &Version, config: &crate::app_config::AppConfig) -> bool {
        match (self, version) {
            (Self::Full(a), b) if a == b => true,
            (Self::Full(user_version), maybe_alias) => {
                match (user_version.alias_name(), maybe_alias.find_aliases(config)) {
                    (None, _) | (_, Err(_)) => false,
                    (Some(user_alias), Ok(aliases)) => {
                        aliases.iter().any(|alias| alias.name() == user_alias)
                    }
                }
            }
            (_, Version::Bypassed | Version::Lts(_) | Version::Alias(_) | Version::Latest) => false,
            (Self::OnlyMajor(major), Version::Semver(other)) => *major == other.major,
            (Self::MajorMinor(major, minor), Version::Semver(other)) => {
                *major == other.major && *minor == other.minor
            }
        }
    }

    /// The conjectrred alias for the user version, if it exists.
    pub fn conjectrred_alias(&self) -> Option<Version> {
        match self {
            UserVersion::Full(Version::Latest) => Some(Version::Latest),
            UserVersion::Full(Version::Lts(lts_type)) => Some(Version::Lts(lts_type.clone())),
            _ => None,
        }
    }
}

fn next_of<'a, T: FromStr, It: Iterator<Item = &'a str>>(i: &mut It) -> Option<T> {
    let x = i.next()?;
    T::from_str(x).ok()
}

impl std::fmt::Display for UserVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full(x) => x.fmt(f),
            Self::OnlyMajor(major) => write!(f, "v{major}.x.x"),
            Self::MajorMinor(major, minor) => write!(f, "v{major}.{minor}.x"),
        }
    }
}

fn skip_first_v(str: &str) -> &str {
    str.strip_prefix('v').unwrap_or(str)
}

impl FromStr for UserVersion {
    type Err = node_semver::SemverError;
    fn from_str(s: &str) -> Result<UserVersion, Self::Err> {
        match Version::parse(s) {
            Ok(v) => Ok(Self::Full(v)),
            Err(e) => {
                let mut parts = skip_first_v(s.trim()).split('.');
                match (next_of::<u64, _>(&mut parts), next_of::<u64, _>(&mut parts)) {
                    (Some(major), None) => Ok(Self::OnlyMajor(major)),
                    (Some(major), Some(minor)) => Ok(Self::MajorMinor(major, minor)),
                    _ => Err(e),
                }
            }
        }
    }
}
