use std::fs;
use std::io;
use std::path::PathBuf;

use image::imageops::FilterType;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Album {
    pub path: PathBuf,
    pub name: String,
    pub author: String,
    pub host_path: String,
    pub readme_path: PathBuf,
    pub images_path: PathBuf,
    pub cover_path: PathBuf,
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
        let mut host_path = try_get_config!(config, "host_path");
        let cover_path = try_get_config!(config, "cover_path").parse().unwrap();
        let readme_path = try_get_config!(config, "readme").parse().unwrap();
        let images_path = try_get_config!(config, "image_dir").parse().unwrap();

        if !host_path.starts_with('/') {
            host_path = format!("/{}", host_path);
        }

        Ok(Album {
            path,
            name,
            author,
            host_path,
            readme_path,
            cover_path,
            images_path,
        })
    }

    pub fn gen_thumbs(&self) -> Option<()> {
        let cache = self.path.join(".thumbs");
        let files = fs::read_dir(self.path.join(self.clone().images_path))
            .ok()?
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();

        if !cache.exists() {
            fs::create_dir(&cache).ok()?;
        }

        for file in files {
            let path = file.path();
            let img = image::open(&path)
                .unwrap()
                .resize(u32::MAX, 255, FilterType::Triangle);
            img.save(cache.join(path.file_name()?)).unwrap();
        }

        Some(())
    }
}

pub fn load_albums<T>(base_path: T) -> Option<Vec<Album>>
where
    T: Into<PathBuf>,
{
    let mut all_files = Vec::new();
    let mut files = fs::read_dir(base_path.into())
        .ok()?
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();

    files.sort_by_key(|x| x.metadata().unwrap().created().unwrap());

    for file in files.iter().rev() {
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
