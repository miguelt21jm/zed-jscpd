use zed_extension_api::{
    self as zed, Command, LanguageServerId, Result, Worktree, serde_json::Value,
};

use crate::{binary_location::BinaryLocation, server_settings::ServerSettings};

/// The extension, holding the path of the binary it downloaded, if any.
struct JscpdExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for JscpdExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let settings = ServerSettings::for_worktree(language_server_id.as_ref(), worktree);
        let location = BinaryLocation::find(
            language_server_id,
            worktree,
            &settings,
            &mut self.cached_binary_path,
        )
        .map_err(|error| error.to_string())?;
        Ok(Command {
            command: location.binary_path,
            args: settings.arguments(),
            env: location.environment,
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        Ok(
            ServerSettings::for_worktree(language_server_id.as_ref(), worktree)
                .initialization_options,
        )
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        Ok(
            ServerSettings::for_worktree(language_server_id.as_ref(), worktree)
                .workspace_configuration,
        )
    }
}

zed::register_extension!(JscpdExtension);
