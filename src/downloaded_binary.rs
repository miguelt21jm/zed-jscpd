use std::{fs, path::Path};

use zed_extension_api::{
    self as zed, DownloadedFileType, GithubRelease, GithubReleaseOptions, LanguageServerId,
    LanguageServerInstallationStatus, Os,
};

use crate::{installation_error::InstallationError, release_asset::ReleaseAsset};

const REPOSITORY: &str = "kucherenko/jscpd";
const FOLDER_PREFIX: &str = "jscpd-";

/// Installs the latest jscpd release into the extension's working directory.
///
/// Each release lives in a `jscpd-<version>` folder; older folders are removed after an
/// install, and the newest one left is used when the release lookup fails offline.
pub fn install(language_server_id: &LanguageServerId) -> Result<String, InstallationError> {
    zed::set_language_server_installation_status(
        language_server_id,
        &LanguageServerInstallationStatus::CheckingForUpdate,
    );
    let installation = install_latest_release(language_server_id);
    let status = match &installation {
        Ok(_) => LanguageServerInstallationStatus::None,
        Err(error) => LanguageServerInstallationStatus::Failed(error.to_string()),
    };
    zed::set_language_server_installation_status(language_server_id, &status);
    installation
}

fn install_latest_release(
    language_server_id: &LanguageServerId,
) -> Result<String, InstallationError> {
    let (operating_system, architecture) = zed::current_platform();
    let asset = ReleaseAsset::for_platform(operating_system, architecture)?;
    let options = GithubReleaseOptions {
        require_assets: true,
        pre_release: false,
    };
    let release = match zed::latest_github_release(REPOSITORY, options) {
        Ok(release) => release,
        Err(reason) => {
            return newest_installed_binary(asset.binary_file_name).ok_or(
                InstallationError::ReleaseLookupFailed {
                    repository: REPOSITORY.to_string(),
                    reason,
                },
            );
        }
    };
    let folder = format!("{FOLDER_PREFIX}{}", release.version);
    let binary_path = format!("{folder}/{}", asset.binary_file_name);
    if !Path::new(&binary_path).exists() {
        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::Downloading,
        );
        download_release(&release, &asset, &folder, &binary_path, operating_system)?;
        remove_folders_other_than(&folder);
    }
    Ok(binary_path)
}

fn download_release(
    release: &GithubRelease,
    asset: &ReleaseAsset,
    folder: &str,
    binary_path: &str,
    operating_system: Os,
) -> Result<(), InstallationError> {
    let download_url = release
        .assets
        .iter()
        .find(|candidate| candidate.name == asset.asset_name)
        .map(|candidate| candidate.download_url.clone())
        .ok_or_else(|| InstallationError::AssetMissing {
            asset_name: asset.asset_name.clone(),
            version: release.version.clone(),
        })?;
    let download = zed::download_file(&download_url, folder, DownloadedFileType::GzipTar)
        .map_err(|reason| InstallationError::DownloadFailed {
            asset_name: asset.asset_name.clone(),
            reason,
        })
        .and_then(|()| make_executable(binary_path, operating_system));
    if download.is_err() {
        let _ = fs::remove_dir_all(folder);
    }
    download
}

fn make_executable(binary_path: &str, operating_system: Os) -> Result<(), InstallationError> {
    if operating_system == Os::Windows {
        return Ok(());
    }
    zed::make_file_executable(binary_path).map_err(|reason| {
        InstallationError::ExecutablePermissionFailed {
            binary_path: binary_path.to_string(),
            reason,
        }
    })
}

fn remove_folders_other_than(kept_folder: &str) {
    for folder in installed_folders() {
        if folder != kept_folder {
            let _ = fs::remove_dir_all(&folder);
        }
    }
}

fn newest_installed_binary(binary_file_name: &str) -> Option<String> {
    installed_folders()
        .into_iter()
        .map(|folder| format!("{folder}/{binary_file_name}"))
        .filter(|binary_path| Path::new(binary_path).exists())
        .max_by_key(|binary_path| version_numbers(binary_path))
}

fn installed_folders() -> Vec<String> {
    fs::read_dir(".")
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|name| name.starts_with(FOLDER_PREFIX))
                .collect()
        })
        .unwrap_or_default()
}

fn version_numbers(binary_path: &str) -> Vec<u64> {
    let folder = binary_path.split('/').next().unwrap_or_default();
    folder
        .trim_start_matches(FOLDER_PREFIX)
        .trim_start_matches('v')
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .unwrap_or_default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_by_number_not_by_text() {
        assert!(version_numbers("jscpd-v5.10.0/jscpd") > version_numbers("jscpd-v5.9.3/jscpd"));
        assert_eq!(version_numbers("jscpd-v5.4.0/jscpd.exe"), vec![5, 4, 0]);
    }

    #[test]
    fn a_prerelease_suffix_keeps_its_numeric_part() {
        assert_eq!(version_numbers("jscpd-5.5.0-rc1/jscpd"), vec![5, 5, 0]);
    }
}
