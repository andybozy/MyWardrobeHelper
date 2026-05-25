use std::env;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_LAN_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 8787;
pub const DEFAULT_DATA_DIR: &str = ".data";

pub const ENV_HOST: &str = "MYWARDROBEHELPER_HOST";
pub const ENV_PORT: &str = "MYWARDROBEHELPER_PORT";
pub const ENV_DATA_DIR: &str = "MYWARDROBEHELPER_DATA_DIR";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub data_dir: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigOverrides {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub data_dir: Option<PathBuf>,
    pub lan: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub data_dir: Option<PathBuf>,
}

impl AppConfig {
    pub fn from_sources(
        overrides: ConfigOverrides,
        env_config: EnvConfig,
        cwd: &Path,
    ) -> AppResult<Self> {
        let explicit_host = overrides.host;
        let host = explicit_host.or(env_config.host).unwrap_or_else(|| {
            if overrides.lan {
                DEFAULT_LAN_HOST.to_string()
            } else {
                DEFAULT_HOST.to_string()
            }
        });

        if host.trim().is_empty() {
            return Err(AppError::config("host cannot be empty"));
        }

        let port = overrides.port.or(env_config.port).unwrap_or(DEFAULT_PORT);
        if port == 0 {
            return Err(AppError::config("port must be greater than 0"));
        }

        let data_dir = overrides
            .data_dir
            .or(env_config.data_dir)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_DIR));
        let data_dir = normalize_path(cwd, data_dir);

        Ok(Self {
            host,
            port,
            data_dir,
        })
    }

    pub fn local_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn bind_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    pub fn lan_url(&self) -> Option<String> {
        if self.host == DEFAULT_LAN_HOST {
            discover_lan_ip().map(|ip| format!("http://{}:{}", ip, self.port))
        } else if is_loopback_host(&self.host) {
            None
        } else {
            Some(self.bind_url())
        }
    }
}

impl EnvConfig {
    pub fn from_process_env() -> AppResult<Self> {
        Ok(Self {
            host: read_env_string(ENV_HOST),
            port: read_env_port(ENV_PORT)?,
            data_dir: env::var_os(ENV_DATA_DIR).map(PathBuf::from),
        })
    }
}

fn read_env_string(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn read_env_port(name: &str) -> AppResult<Option<u16>> {
    let Some(raw) = env::var(name).ok() else {
        return Ok(None);
    };

    let port = raw.parse::<u16>().map_err(|_| {
        AppError::config(format!(
            "environment variable {name} must be a valid TCP port"
        ))
    })?;

    if port == 0 {
        return Err(AppError::config(format!(
            "environment variable {name} must be greater than 0"
        )));
    }

    Ok(Some(port))
}

fn normalize_path(cwd: &Path, candidate: PathBuf) -> PathBuf {
    if candidate.is_absolute() {
        candidate
    } else {
        cwd.join(candidate)
    }
}

fn discover_lan_ip() -> Option<IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let address = socket.local_addr().ok()?.ip();

    if address.is_loopback() {
        None
    } else {
        Some(address)
    }
}

fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_defaults_when_no_overrides_exist() {
        let cwd = Path::new("/tmp/mywardrobehelper-tests");
        let config = AppConfig::from_sources(ConfigOverrides::default(), EnvConfig::default(), cwd)
            .expect("default configuration should parse");

        assert_eq!(config.host, DEFAULT_HOST);
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.data_dir, cwd.join(DEFAULT_DATA_DIR));
    }

    #[test]
    fn lan_flag_changes_default_bind_host() {
        let cwd = Path::new("/tmp/mywardrobehelper-tests");
        let overrides = ConfigOverrides {
            lan: true,
            ..ConfigOverrides::default()
        };

        let config =
            AppConfig::from_sources(overrides, EnvConfig::default(), cwd).expect("config parses");

        assert_eq!(config.host, DEFAULT_LAN_HOST);
    }

    #[test]
    fn explicit_data_dir_is_resolved_from_cwd() {
        let cwd = Path::new("/tmp/mywardrobehelper-tests");
        let overrides = ConfigOverrides {
            data_dir: Some(PathBuf::from("custom-data")),
            ..ConfigOverrides::default()
        };

        let config =
            AppConfig::from_sources(overrides, EnvConfig::default(), cwd).expect("config parses");

        assert_eq!(config.data_dir, cwd.join("custom-data"));
    }
}
