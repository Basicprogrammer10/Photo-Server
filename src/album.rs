use std::fs;
use std::io;
use std::path::PathBuf;

use simple_config_parser::Config;

macro_rules! try_get_config {
    ($config:expr, $key:expr) => {{
        match $config.get_str($key) {
            Ok(i) => i,
            Err(e) => return Err(AlbumError::ConfigParseError(e)),
        }
    }};
}

#[derive(Debug)]
pub enum AlbumError {
    ConfigError(io::Error),
    ConfigParseError(simple_config_parser::ConfigError),
}

#[derive(Debug, Clone)]
pub struct Album {
    pub path: PathBuf,
    pub name: String,
    pub author: String,
    pub host_path: String,
    pub readme_path: PathBuf,
    pub images_path: PathBuf,
}

impl Album {
    pub fn load<T>(path: T) -> Result<Self, AlbumError>
    where
        T: Into<PathBuf>,
    {
        let path = path.into();
        let config = match fs::read_to_string(path.join("config.cfg")) {
            Ok(i) => i,
            Err(e) => return Err(AlbumError::ConfigError(e)),
        };

        let config = match Config::new().text(config) {
            Ok(i) => i,
            Err(e) => return Err(AlbumError::ConfigParseError(e)),
        };

        let name = try_get_config!(config, "name");
        let author = try_get_config!(config, "author");
        let host_path = try_get_config!(config, "host_path");
        let readme_path = try_get_config!(config, "readme").parse().unwrap();
        let image_dir = try_get_config!(config, "image_dir").parse().unwrap();

        Ok(Album {
            path,
            name,
            author,
            host_path,
            readme_path,
            images_path: image_dir,
        })
    }
}

pub fn load_albums<T>(base_path: T) -> Option<Vec<Album>>
where
    T: Into<PathBuf>,
{
    let mut all_files = Vec::new();
    let files = fs::read_dir(base_path.into()).ok()?;

    for file in files {
        let file = file.ok()?;

        if !file.file_name().to_str()?.starts_with("album_") {
            continue;
        }

        if !file.file_type().ok()?.is_dir() {
            continue;
        }

        all_files.push(Album::load(file.path()).ok()?);
    }

    Some(all_files)
}
