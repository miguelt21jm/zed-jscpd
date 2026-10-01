use zed_extension_api::{Architecture, Os};

use crate::installation_error::InstallationError;

/// The release archive for one platform and the binary inside it.
#[derive(Debug, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub asset_name: String,
    pub binary_file_name: &'static str,
}

impl ReleaseAsset {
    /// Linux always maps to the glibc build, since Zed does not report the C library.
    pub fn for_platform(
        operating_system: Os,
        architecture: Architecture,
    ) -> Result<Self, InstallationError> {
        let target = match (operating_system, architecture) {
            (Os::Mac, Architecture::Aarch64) => "darwin-arm64",
            (Os::Mac, Architecture::X8664) => "darwin-x64",
            (Os::Linux, Architecture::Aarch64) => "linux-arm64-gnu",
            (Os::Linux, Architecture::X8664) => "linux-x64-gnu",
            (Os::Windows, Architecture::Aarch64) => "windows-arm64-msvc",
            (Os::Windows, Architecture::X8664) => "windows-x64-msvc",
            (_, Architecture::X86) => {
                return Err(InstallationError::UnsupportedArchitecture {
                    operating_system: format!("{operating_system:?}"),
                    architecture: format!("{architecture:?}"),
                });
            }
        };
        let binary_file_name = match operating_system {
            Os::Windows => "jscpd.exe",
            Os::Mac | Os::Linux => "jscpd",
        };
        Ok(Self {
            asset_name: format!("jscpd-{target}.tar.gz"),
            binary_file_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset_name(operating_system: Os, architecture: Architecture) -> String {
        ReleaseAsset::for_platform(operating_system, architecture)
            .unwrap()
            .asset_name
    }

    #[test]
    fn every_supported_platform_has_its_archive() {
        assert_eq!(
            asset_name(Os::Mac, Architecture::Aarch64),
            "jscpd-darwin-arm64.tar.gz"
        );
        assert_eq!(
            asset_name(Os::Mac, Architecture::X8664),
            "jscpd-darwin-x64.tar.gz"
        );
        assert_eq!(
            asset_name(Os::Linux, Architecture::Aarch64),
            "jscpd-linux-arm64-gnu.tar.gz"
        );
        assert_eq!(
            asset_name(Os::Linux, Architecture::X8664),
            "jscpd-linux-x64-gnu.tar.gz"
        );
        assert_eq!(
            asset_name(Os::Windows, Architecture::Aarch64),
            "jscpd-windows-arm64-msvc.tar.gz"
        );
        assert_eq!(
            asset_name(Os::Windows, Architecture::X8664),
            "jscpd-windows-x64-msvc.tar.gz"
        );
    }

    #[test]
    fn only_windows_binaries_carry_the_exe_extension() {
        let windows = ReleaseAsset::for_platform(Os::Windows, Architecture::X8664).unwrap();
        let linux = ReleaseAsset::for_platform(Os::Linux, Architecture::X8664).unwrap();
        assert_eq!(windows.binary_file_name, "jscpd.exe");
        assert_eq!(linux.binary_file_name, "jscpd");
    }

    #[test]
    fn x86_is_unsupported_on_every_system() {
        for operating_system in [Os::Mac, Os::Linux, Os::Windows] {
            let error =
                ReleaseAsset::for_platform(operating_system, Architecture::X86).unwrap_err();
            assert!(matches!(
                error,
                InstallationError::UnsupportedArchitecture { .. }
            ));
        }
    }
}
