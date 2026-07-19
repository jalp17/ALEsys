//! Security Sandbox for plugin execution

use super::api::PluginPermission;

/// Sandboxed execution environment for plugins
pub struct SecuritySandbox {
    #[allow(dead_code)]
    permissions: Vec<PluginPermission>,
}

impl SecuritySandbox {
    /// Create a new sandbox with given permissions
    pub fn new(permissions: Vec<PluginPermission>) -> Self {
        Self { permissions }
    }

    /// Check if a permission is granted
    pub fn has_permission(&self, permission: &PluginPermission) -> bool {
        self.permissions.iter().any(|p| std::mem::discriminant(p) == std::mem::discriminant(permission))
    }

    /// Validate path is within allowed directories
    pub fn validate_path(&self, path: &str) -> bool {
        for perm in &self.permissions {
            if let PluginPermission::FilesystemRead { allowed_paths }
            | PluginPermission::FilesystemWrite { allowed_paths } = perm
            {
                for allowed in allowed_paths {
                    if path.starts_with(allowed) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Validate command is allowed
    pub fn validate_command(&self, command: &str) -> bool {
        for perm in &self.permissions {
            if let PluginPermission::Execute { allowed_commands } = perm {
                for allowed in allowed_commands {
                    if command.starts_with(allowed) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Validate network host is allowed
    pub fn validate_host(&self, host: &str) -> bool {
        for perm in &self.permissions {
            if let PluginPermission::Network { allowed_hosts } = perm {
                for allowed in allowed_hosts {
                    if host == allowed || host.ends_with(&format!(".{}", allowed)) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
