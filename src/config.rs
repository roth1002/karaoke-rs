extern crate config as cfg;

use dirs::{config_dir, data_dir};
use karaoke::embed::create_config_if_not_exists;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{default::Default, fs::DirBuilder, path::PathBuf};

//Default locations, overriden if supplied in Config file or by Argument
lazy_static! {
    pub static ref CONF_FILE: PathBuf = {
        let mut config_dir = config_dir().unwrap();
        config_dir.push("karaoke-rs");
        DirBuilder::new()
            .recursive(true)
            .create(config_dir.clone())
            .unwrap();

        let mut path = config_dir;
        path.push("config.yaml");
        path
    };
    pub static ref DATA_DIR: PathBuf = {
        let mut dir = data_dir().unwrap();
        dir.push("karaoke-rs");
        DirBuilder::new()
            .recursive(true)
            .create(dir.clone())
            .unwrap();
        dir
    };
    pub static ref SONG_DIR: PathBuf = {
        let mut dir = DATA_DIR.to_path_buf();
        dir.push("songs");
        dir
    };
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub song_path: PathBuf,
    pub data_path: PathBuf,
    pub no_collection_update: bool,
    pub use_web_player: bool,
    pub port: u16,
    pub port_ws: u16,
    pub song_format: String,
    pub player: PlayerConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            song_path: SONG_DIR.to_path_buf(),
            data_path: DATA_DIR.to_path_buf(),
            no_collection_update: false,
            use_web_player: false,
            port: 8080,
            port_ws: 9000,
            song_format: "[*] - [Artist] - [Title]".to_owned(),
            player: PlayerConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PlayerConfig {
    pub fullscreen: bool,
    pub scale: f32,
    pub disable_background: bool,
}

impl Default for PlayerConfig {
    fn default() -> PlayerConfig {
        PlayerConfig {
            fullscreen: false,
            scale: 1.5,
            disable_background: false,
        }
    }
}

//Use default config or override with valid values from file
fn default_or_file(config_path: PathBuf) -> Result<Config, failure::Error> {
    let mut _config = cfg::Config::new();

    //Serialize from default config struct
    _config.merge(cfg::Config::try_from(&Config::default())?)?;

    //If config file exists, merge and overwrite for values that exist
    if config_path.is_file() {
        let file = cfg::File::from(config_path).format(cfg::FileFormat::Yaml);
        _config.merge(file)?;
    }

    //Deserialize back to Config struct
    let config: Config = _config.try_into()?;

    Ok(config)
}

//Loads a configuration file from default / supplied path, then overrides contents with any supplied args
pub fn load_config(
    config_path: Option<PathBuf>,
    song_path: Option<PathBuf>,
    data_path: Option<PathBuf>,
    refresh_collection: Option<bool>,
    use_web_player: Option<bool>,
    port: Option<u16>,
    port_ws: Option<u16>,
) -> Result<Config, failure::Error> {
    //If config_path supplied (from Arg), use that over default location
    let config_file: PathBuf;
    match config_path {
        Some(path) => {
            config_file = path;
        }
        None => {
            config_file = CONF_FILE.to_path_buf();
        }
    }
    log::info!("Using config file: {:?}", config_file.display());

    //Write config template to path, if not exists
    create_config_if_not_exists(&config_file)?;

    //get Config struct from db
    let mut config = default_or_file(config_file)?;

    //Update config with supplied Args
    if let Some(path) = song_path {
        config.song_path = path;
    }
    if let Some(path) = data_path {
        config.data_path = path;
    }
    if let Some(bool) = refresh_collection {
        config.no_collection_update = !bool;
    }
    if let Some(bool) = use_web_player {
        config.use_web_player = bool;
    }
    if let Some(port) = port {
        config.port = port;
    }
    if let Some(port) = port_ws {
        config.port_ws = port;
    }
    log::info!("Using song dir: {:?}", config.song_path);
    log::info!("Using data dir: {:?}", config.data_path);
    log::info!(
        "Collection to be refreshed: {:?}",
        !config.no_collection_update
    );
    log::info!("Use web player: {:?}", config.use_web_player);

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::remove_file, path::PathBuf};

    #[test]
    fn test_create_default_config() {
        let config_path = PathBuf::from("tests/test_data/config.yaml");
        assert!(!config_path.is_file());
        let config = load_config(
            Some(config_path.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert!(config_path.is_file());
        assert_eq!(config, Config::default());

        remove_file("tests/test_data/config.yaml").unwrap();
    }

    //Song & Data path already checked to be valid directories before passing to
    //load_config function, don't need to test it here.
    #[test]
    fn test_create_custom_config() {
        let config_path = PathBuf::from("tests/test_data/config.yaml");
        let song_path = PathBuf::from("test/test_data/songs");
        let data_path = PathBuf::from("test/test_data");
        let config = load_config(
            Some(config_path),
            Some(song_path),
            Some(data_path),
            Some(true),
            Some(true),
            Some(8000),
            Some(9090),
        )
        .unwrap();
        let _config = Config {
            song_path: PathBuf::from("test/test_data/songs"),
            data_path: PathBuf::from("test/test_data"),
            use_web_player: true,
            port: 8000,
            port_ws: 9090,
            ..Config::default()
        };
        assert_eq!(config, _config);

        remove_file("tests/test_data/config.yaml").unwrap();
    }
}
