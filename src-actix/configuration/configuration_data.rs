use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

static CONFIGURATION: OnceLock<Configuration> = OnceLock::new();
static CONFIGURATION_PATH: OnceLock<Option<String>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub port: u16,
    pub root_path: String,
    pub indexing_enabled: bool,
    pub file_watcher_enabled: bool,
    pub filter_mode_whitelist: bool,
    pub filter: Vec<String>,
    pub included_extensions: Vec<String>,
    pub exclude_hidden_files: bool,
    pub upnp_enabled: bool,
    /// A list of authorized hosts for the server, allowing access from these hosts only.
    /// By default, it includes localhost, 127.0.0.1 and the server's own IP address.
    pub authorized_hosts: Vec<String>,
    pub cors_enabled: bool,
}

impl Configuration {
    pub fn get() -> &'static Self {
        if let Some(config) = CONFIGURATION.get() {
            config
        } else {
            CONFIGURATION.set(Self::default()).expect("Failed to set default configuration in OnceLock");
            CONFIGURATION.get().expect("Failed to get configuration from OnceLock after setting default")
        }
    }
    pub fn get_path() -> &'static Option<String> {
        CONFIGURATION_PATH.get().expect("Configuration path OnceLock not initialized")
    }
    pub fn set_path(path: impl AsRef<Path>) -> anyhow::Result<()> {
        debug!("Setting configuration path to {:?}", path.as_ref());
        CONFIGURATION_PATH.set(Some(path.as_ref().to_string_lossy().to_string())).map_err(|_| anyhow::anyhow!("Failed to set configuration path"))?;
        Ok(())
    }
    pub fn load() -> anyhow::Result<Self> {
        debug!("Loading configuration");
        let path = Self::get_path().clone().ok_or_else(|| anyhow::anyhow!("Configuration path is not set"))?;
        if !Path::new(path.as_str()).exists() {
            warn!("Configuration file does not exist, resetting to default");
            Self::reset()?;
            return Ok(Self::default());
        }
        let file_contents = std::fs::read_to_string(path)?;
        if let Ok(config) = serde_json::from_str::<Configuration>(file_contents.as_str()) {
            debug!("Configuration loaded successfully");
            if CONFIGURATION.set(config.clone()).is_err() {
                warn!("Failed to set configuration in global state, using default");
            }
            Ok(config)
        } else {
            warn!("Configuration file is invalid, resetting to default");
            Self::reset()?;
            Ok(Self::default())
        }
    }
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::get_path().clone().ok_or_else(|| anyhow::anyhow!("Configuration path is not set"))?;
        debug!("Saving configuration to {:?}", path);
        let file_contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, file_contents)?;
        Ok(())
    }
    pub fn reset() -> anyhow::Result<()> {
        debug!("Resetting configuration");
        Self::default().save()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        // Get the current executable path, falling back to empty string if it fails
        let current_exe_path =
            std::env::current_exe().ok().and_then(|path| path.parent().map(|p| p.to_string_lossy().to_string())).unwrap_or_else(|| {
                warn!("Failed to get current executable path, using empty string");
                String::new()
            });
        let current_exe_path = current_exe_path.as_str().replace('\\', "/");

        // Get the current working directory, falling back to an empty string if it fails
        let cwd = std::env::current_dir().map(|path| path.to_string_lossy().to_string()).unwrap_or_else(|_| {
            warn!("Failed to get current working directory, using empty string");
            String::new()
        });
        let cwd = cwd.as_str().replace('\\', "/");
        let cwd = format!("{}/**/*", cwd);
        let current_exe_path = format!("{}/**/*", current_exe_path);
        let mut ignored_paths = vec![
            "/dev/**/*",
            "/proc/**/*",
            "/sys/**/*",
            "/run/**/*",
            "/mnt/**/*",
            "/media/**/*",
            "/lost+found/**/*",
            "/var/log/**/*",
            "/var/cache/**/*",
            "C:/Windows/**/*",
            "C:/Windows.old/**/*",
            "C:/Program Files/Windows Defender/**/*",
            "C:/ProgramData/Microsoft/**/*",
            "C:/System Volume Information/**/*",
            "C:/Recovery/**/*",
            "C:/PerfLogs/**/*",
            "C:/Users/*/AppData/Local/Temp/**/*",
            "C:/Users/*/AppData/LocalLow/**/*",
            "C:/Users/*/AppData/Local/Microsoft/**/*",
            "**/*.log",
            "**/*.db*",
            "**/*.dat",
            "**/*.lock",
            "**/*.tmp",
            "**/*.bak",
            "**/Temp/**",
            "**/Tmp/**",
            "**/tmp/**",
            "**/temp/**",
            current_exe_path.as_str(),
        ];

        if current_exe_path != cwd {
            ignored_paths.push(cwd.as_str())
        }

        let ignored_paths = ignored_paths.into_iter().map(String::from).collect();
        let server_computer_ip_address = if let Some(ip) = local_ipaddress::get() { ip } else { "".to_string() };

        Self {
            port: 7667,
            root_path: "/".to_string(),
            indexing_enabled: true,
            file_watcher_enabled: true,
            filter_mode_whitelist: false,
            filter: ignored_paths,
            included_extensions: vec![
                ".txt".to_string(),
                ".pdf".to_string(),
                ".doc".to_string(),
                ".docx".to_string(),
                ".jpg".to_string(),
                ".png".to_string(),
                ".mp4".to_string(),
                ".mp3".to_string(),
            ],
            exclude_hidden_files: true,
            // Network defaults
            upnp_enabled: false,
            authorized_hosts: vec!["127.0.0.1".to_string(), "localhost".to_string(), server_computer_ip_address],
            cors_enabled: true,
        }
    }
}
