use std::path::Path;

use zed_extension_api::{EnvVars, LanguageServerId, Worktree};

use crate::{
    downloaded_binary, installation_error::InstallationError, server_settings::ServerSettings,
};

const BINARY_NAME: &str = "jscpd";

/// The jscpd binary to start and the environment to start it in.
pub struct BinaryLocation {
    pub binary_path: String,
    pub environment: EnvVars,
}

impl BinaryLocation {
    /// Looks in order at the configured path, `PATH`, the cached download and a new download.
    pub fn find(
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
        settings: &ServerSettings,
        cached_binary_path: &mut Option<String>,
    ) -> Result<Self, InstallationError> {
        let system_binary_path = settings
            .binary_path
            .clone()
            .or_else(|| worktree.which(BINARY_NAME));
        if let Some(binary_path) = system_binary_path {
            return Ok(Self {
                binary_path,
                environment: settings.environment_over(worktree.shell_env()),
            });
        }
        let downloaded_binary_path = match cached_binary_path
            .clone()
            .filter(|binary_path| Path::new(binary_path).exists())
        {
            Some(binary_path) => binary_path,
            None => downloaded_binary::install(language_server_id)?,
        };
        *cached_binary_path = Some(downloaded_binary_path.clone());
        Ok(Self {
            binary_path: downloaded_binary_path,
            environment: settings.environment_over(Vec::new()),
        })
    }
}
