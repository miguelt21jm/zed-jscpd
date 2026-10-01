use std::fmt;

const INSTALL_ADVICE: &str =
    "Install jscpd 5.4.0 or later on PATH, or set lsp.jscpd.binary.path in the Zed settings.";

/// A reason the jscpd binary could not be found or installed.
#[derive(Debug, PartialEq, Eq)]
pub enum InstallationError {
    UnsupportedArchitecture {
        operating_system: String,
        architecture: String,
    },
    ReleaseLookupFailed {
        repository: String,
        reason: String,
    },
    AssetMissing {
        asset_name: String,
        version: String,
    },
    DownloadFailed {
        asset_name: String,
        reason: String,
    },
    ExecutablePermissionFailed {
        binary_path: String,
        reason: String,
    },
}

impl fmt::Display for InstallationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedArchitecture {
                operating_system,
                architecture,
            } => write!(
                formatter,
                "jscpd publishes no binary for {operating_system} on {architecture}. {INSTALL_ADVICE}"
            ),
            Self::ReleaseLookupFailed { repository, reason } => write!(
                formatter,
                "Could not look up the latest release of {repository}: {reason}. Check the network connection. {INSTALL_ADVICE}"
            ),
            Self::AssetMissing {
                asset_name,
                version,
            } => write!(
                formatter,
                "jscpd {version} has no release asset named {asset_name}. {INSTALL_ADVICE}"
            ),
            Self::DownloadFailed { asset_name, reason } => write!(
                formatter,
                "Could not download {asset_name}: {reason}. Check the network connection. {INSTALL_ADVICE}"
            ),
            Self::ExecutablePermissionFailed {
                binary_path,
                reason,
            } => write!(
                formatter,
                "Could not make {binary_path} executable: {reason}. {INSTALL_ADVICE}"
            ),
        }
    }
}
