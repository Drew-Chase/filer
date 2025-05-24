use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

static CONFIGURATION: OnceLock<Configuration> = OnceLock::new();
static CONFIGURATION_PATH: OnceLock<Option<String>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub port: u16,
    pub indexing_enabled: bool,
    pub file_watcher_enabled: bool,
    pub filter_mode_whitelist: bool,
    pub filter: Vec<String>,
}

impl Configuration {
    pub fn get() -> &'static Self {
        if let Some(config) = CONFIGURATION.get() {
            config
        } else {
            CONFIGURATION.set(Self::default()).unwrap();
            CONFIGURATION.get().unwrap()
        }
    }
    pub fn get_path() -> &'static Option<String> {
        CONFIGURATION_PATH.get().unwrap()
    }
    pub fn set_path(path: impl AsRef<Path>) -> anyhow::Result<()> {
        debug!("Setting configuration path to {:?}", path.as_ref());
        CONFIGURATION_PATH
            .set(Some(path.as_ref().to_string_lossy().to_string()))
            .map_err(|_| anyhow::anyhow!("Failed to set configuration path"))?;
        Ok(())
    }
    pub fn load() -> anyhow::Result<Self> {
        debug!("Loading configuration");
        let path = Self::get_path()
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Configuration path is not set"))?;
        if !Path::new(path.as_str()).exists() {
            warn!("Configuration file does not exist, resetting to default");
            Self::reset()?;
            return Ok(Self::default());
        }
        let file_contents = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(file_contents.as_str())?)
    }
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::get_path()
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Configuration path is not set"))?;
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
        let current_exe_path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let current_exe_path = current_exe_path.as_str().replace('\\', "/");
        let cwd = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let cwd = cwd.as_str().replace('\\', "/");
        let cwd = format!("{}/**/*", cwd);
        let current_exe_path = format!("{}/**/*", current_exe_path);
        let ignored_paths = vec![
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
            cwd.as_str(),
        ];

        let ignored_paths = ignored_paths.into_iter().map(String::from).collect();

        Self {
            port: 7667,
            indexing_enabled: true,
            file_watcher_enabled: true,
            filter_mode_whitelist: false,
            filter: ignored_paths,
        }
    }
}
