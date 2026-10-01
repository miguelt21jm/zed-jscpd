use zed_extension_api::{EnvVars, Worktree, serde_json::Value, settings::LspSettings};

const LSP_ARGUMENT: &str = "--lsp";

/// The `lsp.jscpd` section of the Zed settings, read for one worktree.
#[derive(Debug, Default, PartialEq)]
pub struct ServerSettings {
    pub binary_path: Option<String>,
    pub extra_arguments: Vec<String>,
    pub extra_environment: EnvVars,
    pub initialization_options: Option<Value>,
    pub workspace_configuration: Option<Value>,
}

impl ServerSettings {
    pub fn for_worktree(language_server_name: &str, worktree: &Worktree) -> Self {
        LspSettings::for_worktree(language_server_name, worktree)
            .map(Self::from_lsp_settings)
            .unwrap_or_default()
    }

    pub fn from_lsp_settings(lsp_settings: LspSettings) -> Self {
        let binary = lsp_settings.binary;
        let binary_path = binary.as_ref().and_then(|binary| binary.path.clone());
        let extra_arguments = binary
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_default();
        let mut extra_environment: EnvVars = binary
            .and_then(|binary| binary.env)
            .map(|environment| environment.into_iter().collect())
            .unwrap_or_default();
        extra_environment.sort();
        Self {
            binary_path,
            extra_arguments,
            extra_environment,
            initialization_options: lsp_settings.initialization_options,
            workspace_configuration: lsp_settings.settings,
        }
    }

    /// `binary.arguments` extends the command rather than replacing it: `--lsp` is put first
    /// unless the arguments already hold it.
    pub fn arguments(&self) -> Vec<String> {
        let lsp_argument = (!self
            .extra_arguments
            .iter()
            .any(|argument| argument == LSP_ARGUMENT))
        .then(|| LSP_ARGUMENT.to_string());
        lsp_argument
            .into_iter()
            .chain(self.extra_arguments.iter().cloned())
            .collect()
    }

    pub fn environment_over(&self, base_environment: EnvVars) -> EnvVars {
        let mut environment: EnvVars = base_environment
            .into_iter()
            .filter(|(name, _)| {
                !self
                    .extra_environment
                    .iter()
                    .any(|(extra_name, _)| extra_name == name)
            })
            .collect();
        environment.extend(self.extra_environment.iter().cloned());
        environment
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use zed_extension_api::{serde_json::json, settings::CommandSettings};

    use super::*;

    fn settings_with_binary(binary: CommandSettings) -> ServerSettings {
        ServerSettings::from_lsp_settings(LspSettings {
            binary: Some(binary),
            initialization_options: None,
            settings: None,
        })
    }

    fn pair(name: &str, value: &str) -> (String, String) {
        (name.to_string(), value.to_string())
    }

    #[test]
    fn without_settings_the_server_runs_with_only_the_lsp_flag() {
        let settings = ServerSettings::from_lsp_settings(LspSettings::default());
        assert_eq!(settings.arguments(), vec!["--lsp"]);
        assert_eq!(settings.binary_path, None);
    }

    #[test]
    fn binary_arguments_follow_the_lsp_flag() {
        let settings = settings_with_binary(CommandSettings {
            path: Some("/opt/jscpd".to_string()),
            arguments: Some(vec!["--lsp-analyses".to_string(), "all".to_string()]),
            env: None,
        });
        assert_eq!(settings.arguments(), vec!["--lsp", "--lsp-analyses", "all"]);
        assert_eq!(settings.binary_path.as_deref(), Some("/opt/jscpd"));
    }

    #[test]
    fn an_explicit_lsp_flag_is_not_repeated() {
        let settings = settings_with_binary(CommandSettings {
            path: None,
            arguments: Some(vec![
                "--min-tokens".to_string(),
                "40".to_string(),
                "--lsp".to_string(),
            ]),
            env: None,
        });
        assert_eq!(settings.arguments(), vec!["--min-tokens", "40", "--lsp"]);
    }

    #[test]
    fn configured_variables_replace_those_of_the_shell() {
        let settings = settings_with_binary(CommandSettings {
            path: None,
            arguments: None,
            env: Some(HashMap::from([pair("RUST_LOG", "debug")])),
        });
        let environment =
            settings.environment_over(vec![pair("PATH", "/usr/bin"), pair("RUST_LOG", "warn")]);
        assert_eq!(
            environment,
            vec![pair("PATH", "/usr/bin"), pair("RUST_LOG", "debug")]
        );
    }

    #[test]
    fn options_and_settings_pass_through_unchanged() {
        let options = json!({ "lsp": { "complexity": { "enabled": true } } });
        let configuration = json!({ "minTokens": 40 });
        let settings = ServerSettings::from_lsp_settings(LspSettings {
            binary: None,
            initialization_options: Some(options.clone()),
            settings: Some(configuration.clone()),
        });
        assert_eq!(settings.initialization_options, Some(options));
        assert_eq!(settings.workspace_configuration, Some(configuration));
    }
}
