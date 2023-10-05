use crate::version::Version;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CPUArch {
    X86,
    X64,
    X64Musl,
    Arm64,
    Armv7l,
    Ppc64le,
    Ppc64,
    S390x,
}

impl CPUArch {
    pub fn as_str(self) -> &'static str {
        match self {
            CPUArch::X86 => "x86",
            CPUArch::X64 => "x64",
            CPUArch::X64Musl => "x64-musl",
            CPUArch::Arm64 => "arm64",
            CPUArch::Armv7l => "armv7l",
            CPUArch::Ppc64le => "ppc64le",
            CPUArch::Ppc64 => "ppc64",
            CPUArch::S390x => "s390x",
        }
    }
}

#[cfg(unix)]
/// handle common case: Apple Silicon / Node < 16
pub fn get_safe_cpu_arch(cpu_arch: CPUArch, version: &Version) -> CPUArch {
    use crate::system_info::{platform_cpu_arch, platform_name};

    match (platform_name(), platform_cpu_arch(), version) {
        ("darwin", "arm64", Version::Semver(v)) if v.major < 16 => CPUArch::X64,
        _ => cpu_arch,
    }
}

#[cfg(windows)]
/// handle common case: Apple Silicon / Node < 16
pub fn get_safe_cpu_arch(cpu_arch: CPUArch, _version: &Version) -> CPUArch {
    cpu_arch
}

impl Default for CPUArch {
    fn default() -> CPUArch {
        match crate::system_info::platform_cpu_arch().parse() {
            Ok(cpu_arch) => cpu_arch,
            Err(e) => panic!("{}", e.details),
        }
    }
}

impl std::str::FromStr for CPUArch {
    type Err = CPUArchError;
    fn from_str(s: &str) -> Result<CPUArch, Self::Err> {
        match s {
            "x86" => Ok(CPUArch::X86),
            "x64" => Ok(CPUArch::X64),
            "x64-musl" => Ok(CPUArch::X64Musl),
            "arm64" => Ok(CPUArch::Arm64),
            "armv7l" => Ok(CPUArch::Armv7l),
            "ppc64le" => Ok(CPUArch::Ppc64le),
            "ppc64" => Ok(CPUArch::Ppc64),
            "s390x" => Ok(CPUArch::S390x),
            unknown => Err(CPUArchError::new(format!("Unknown CPUArch: {unknown}"))),
        }
    }
}

impl std::fmt::Display for CPUArch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct CPUArchError {
    details: String,
}

impl CPUArchError {
    fn new(msg: String) -> CPUArchError {
        CPUArchError { details: msg }
    }
}

impl std::fmt::Display for CPUArchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.details)
    }
}

impl std::error::Error for CPUArchError {
    fn description(&self) -> &str {
        &self.details
    }
}
