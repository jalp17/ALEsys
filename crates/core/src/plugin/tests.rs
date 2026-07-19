#[cfg(test)]
mod tests {
    use crate::plugin::api::{Plugin, PluginContext, PluginPermission};
    use crate::plugin::manager::{GitIntegrationPlugin, TestRunnerPlugin, DockerRunnerPlugin};
    use crate::plugin::security::SecuritySandbox;
    use std::path::PathBuf;
    use std::collections::HashMap;

    fn test_context() -> PluginContext {
        PluginContext {
            work_dir: PathBuf::from("/tmp"),
            allowed_paths: vec![],
            config: HashMap::new(),
            request_id: "test-123".to_string(),
        }
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = GitIntegrationPlugin::new();
        let meta = plugin.metadata();
        assert_eq!(meta.id, "git-integration");
        assert_eq!(meta.name, "Git Integration");
        assert_eq!(meta.version, "0.1.0");
    }

    #[test]
    fn test_plugin_can_handle() {
        let plugin = GitIntegrationPlugin::new();
        assert!(plugin.can_handle("git.status"));
        assert!(plugin.can_handle("git.diff"));
        assert!(!plugin.can_handle("test.run"));
    }

    #[test]
    fn test_plugin_supported_commands() {
        let plugin = GitIntegrationPlugin::new();
        let commands = plugin.supported_commands();
        assert!(commands.contains(&"git.status".to_string()));
        assert!(commands.contains(&"git.diff".to_string()));
        assert!(commands.contains(&"git.log".to_string()));
    }

    #[test]
    fn test_plugin_execute() {
        let plugin = GitIntegrationPlugin::new();
        let ctx = test_context();
        let result = plugin.execute("git.status", &[], &ctx).unwrap();
        assert!(result.success);
        assert!(result.output.is_some());
    }

    #[test]
    fn test_plugin_init() {
        let mut plugin = GitIntegrationPlugin::new();
        let ctx = test_context();
        assert!(plugin.init(&ctx).is_ok());
    }

    #[test]
    fn test_plugin_shutdown() {
        let mut plugin = GitIntegrationPlugin::new();
        assert!(plugin.shutdown().is_ok());
    }

    #[test]
    fn test_security_sandbox_validate_path() {
        let sandbox = SecuritySandbox::new(vec![PluginPermission::FilesystemRead {
            allowed_paths: vec!["/tmp".to_string(), "/home/user".to_string()],
        }]);
        assert!(sandbox.validate_path("/tmp/test.txt"));
        assert!(sandbox.validate_path("/home/user/file.rs"));
        assert!(!sandbox.validate_path("/etc/passwd"));
    }

    #[test]
    fn test_security_sandbox_validate_command() {
        let sandbox = SecuritySandbox::new(vec![PluginPermission::Execute {
            allowed_commands: vec!["git".to_string(), "cargo".to_string()],
        }]);
        assert!(sandbox.validate_command("git status"));
        assert!(sandbox.validate_command("cargo test"));
        assert!(!sandbox.validate_command("rm -rf /"));
    }

    #[test]
    fn test_security_sandbox_validate_host() {
        let sandbox = SecuritySandbox::new(vec![PluginPermission::Network {
            allowed_hosts: vec!["github.com".to_string(), "crates.io".to_string()],
        }]);
        assert!(sandbox.validate_host("github.com"));
        assert!(sandbox.validate_host("api.github.com"));
        assert!(!sandbox.validate_host("evil.com"));
    }

    #[test]
    fn test_test_runner_plugin() {
        let plugin = TestRunnerPlugin::new();
        let meta = plugin.metadata();
        assert_eq!(meta.id, "test-runner");
        let ctx = test_context();
        let result = plugin.execute("test.run", &[], &ctx).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_docker_runner_plugin() {
        let plugin = DockerRunnerPlugin::new();
        let meta = plugin.metadata();
        assert_eq!(meta.id, "docker-runner");
        let ctx = test_context();
        let result = plugin.execute("docker.run", &[], &ctx).unwrap();
        assert!(result.success);
    }
}
