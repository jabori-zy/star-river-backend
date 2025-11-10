use exchange_core::ExchangeMetadata;

// ============================================================================
// Mt5Metadata Structure
// ============================================================================

/// MT5 exchange metadata
///
/// Stores MT5-specific configuration and runtime information
#[derive(Debug)]
pub struct Mt5Metadata {
    server: String,
    terminal_id: i32,
    login: i64,
    password: String,
    terminal_path: String,
}

impl Mt5Metadata {
    /// Create new MT5 metadata
    pub fn new(server: String, terminal_id: i32, login: i64, password: String, terminal_path: String) -> Self {
        Self {
            server,
            terminal_id,
            login,
            password,
            terminal_path,
        }
    }

    /// Get server name
    pub fn server(&self) -> &str {
        &self.server
    }

    /// Get terminal ID
    pub fn terminal_id(&self) -> i32 {
        self.terminal_id
    }

    /// Get login account
    pub fn login(&self) -> i64 {
        self.login
    }

    /// Get password
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Get terminal path
    pub fn terminal_path(&self) -> &str {
        &self.terminal_path
    }

    /// Set server name
    pub fn set_server(&mut self, server: String) {
        self.server = server;
    }

    /// Set terminal ID
    pub fn set_terminal_id(&mut self, terminal_id: i32) {
        self.terminal_id = terminal_id;
    }

    /// Set login account
    pub fn set_login(&mut self, login: i64) {
        self.login = login;
    }

    /// Set password
    pub fn set_password(&mut self, password: String) {
        self.password = password;
    }

    /// Set terminal path
    pub fn set_terminal_path(&mut self, terminal_path: String) {
        self.terminal_path = terminal_path;
    }
}

impl ExchangeMetadata for Mt5Metadata {}
