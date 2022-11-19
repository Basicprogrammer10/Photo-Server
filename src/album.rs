use std::collections::hash_map::DefaultHasher;
use std::fs::{self, DirEntry};
use std::hash::{Hash, Hasher};
use std::io;
use std::path::PathBuf;

use fast_image_resize as fir;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use simple_config_parser::Config;

const THUMBNAIL_SIZE: u32 = 255;
const PREVIEW_SIZE: (u32, u32) = (1920, 1080);

lazy_static! {
    static ref END_NUM_REGEX: Regex = Regex::new(r#"[0-9]+\..+"#).unwrap();
    static ref PROGRESS_STYLE: ProgressStyle =
        ProgressStyle::with_template(" └── [{bar:50}] ETA: {eta}, {per_sec}").unwrap();
}

macro_rules! try_get_config {
    ($config:expr, $key:expr) => {{
        match $config.get_str($key) {
            Ok(i) => i,
            Err(e) => return Err(AlbumError::ConfigParse(e)),
        }
    }};
}

#[derive(Debug)]
pub enum AlbumError {
    Config(io::Error),
    ConfigParse(simple_config_parser::ConfigError),
    Io(),
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
    pub images: usize,
}

impl Album {
    pub fn load<T>(path: T) -> Result<Self, AlbumError>
    where
        T: Into<PathBuf>,
    {
        let path = path.into();
        let config = match fs::read_to_string(path.join("config.cfg")) {
            Ok(i) => i,
            Err(e) => return Err(AlbumError::Config(e)),
        };

        let config = match Config::new().text(config) {
            Ok(i) => i,
            Err(e) => return Err(AlbumError::ConfigParse(e)),
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

        let images = match fs::read_dir(path.join(&images_path)) {
            Ok(i) => i.count(),
            Err(_) => return Err(AlbumError::Io()),
        };

        Ok(Album {
            path,
            name,
            author,
            host_path,
            readme_path,
            cover_path,
            images_path,
            images,
        })
    }

    pub fn gen_thumbs(&self) -> Option<()> {
        let cache = self.path.join(".thumbs");
        let mut files = fs::read_dir(self.path.join(self.clone().images_path))
            .ok()?
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        sort_photos(&mut files);

        if !cache.exists() {
            fs::create_dir(&cache).ok()?;
        }

        let mut s = DefaultHasher::new();
        files
            .iter()
            .map(|x| x.path())
            .collect::<Vec<PathBuf>>()
            .hash(&mut s);
        let hash = s.finish();

        let bar = ProgressBar::new(files.len() as u64);
        bar.set_style(PROGRESS_STYLE.to_owned());

        for file in files {
            let path = file.path();
            let img = image::open(&path).unwrap();
            let dst_image = crate::image::scale_image(
                &img,
                u32::MAX,
                THUMBNAIL_SIZE,
                fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3),
            );

            image::save_buffer(
                cache.join(path.file_name()?),
                dst_image.buffer(),
                dst_image.width().get(),
                dst_image.height().get(),
                img.color(),
            )
            .unwrap();
            bar.inc(1);
        }

        fs::write(cache.join(".lock"), format!("{:x}", hash)).ok()?;

        Some(())
    }

    pub fn check_thumbs(&self) -> Option<bool> {
        let cache = self.path.join(".thumbs");
        let mut files = fs::read_dir(self.path.join(self.clone().images_path))
            .ok()?
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        sort_photos(&mut files);

        let lock = match fs::read_to_string(cache.join(".lock")) {
            Ok(i) => i,
            Err(_) => return Some(false),
        };

        let mut s = DefaultHasher::new();
        files
            .iter()
            .map(|x| x.path())
            .collect::<Vec<PathBuf>>()
            .hash(&mut s);
        let hash = s.finish();

        if lock == format!("{:x}", hash) {
            return Some(true);
        }

        Some(false)
    }

    pub fn gen_previews(&self) -> Option<()> {
        let cache = self.path.join(".previews");
        let mut files = fs::read_dir(self.path.join(self.clone().images_path))
            .ok()?
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        sort_photos(&mut files);

        if !cache.exists() {
            fs::create_dir(&cache).ok()?;
        }

        let mut s = DefaultHasher::new();
        files
            .iter()
            .map(|x| x.path())
            .collect::<Vec<PathBuf>>()
            .hash(&mut s);
        let hash = s.finish();

        let bar = ProgressBar::new(files.len() as u64);
        bar.set_style(PROGRESS_STYLE.to_owned());
        for file in files {
            let path = file.path();
            let img = image::open(&path).unwrap();
            let dst_image = crate::image::scale_image(
                &img,
                PREVIEW_SIZE.0,
                PREVIEW_SIZE.1,
                fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3),
            );

            image::save_buffer(
                cache.join(path.file_name()?),
                dst_image.buffer(),
                dst_image.width().get(),
                dst_image.height().get(),
                img.color(),
            )
            .unwrap();
            bar.inc(1);
        }

        fs::write(cache.join(".lock"), format!("{:x}", hash)).ok()?;

        Some(())
    }

    pub fn check_previews(&self) -> Option<bool> {
        let cache = self.path.join(".previews");
        let mut files = fs::read_dir(self.path.join(self.clone().images_path))
            .ok()?
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        sort_photos(&mut files);

        let lock = match fs::read_to_string(cache.join(".lock")) {
            Ok(i) => i,
            Err(_) => return Some(false),
        };

        let mut s = DefaultHasher::new();
        files
            .iter()
            .map(|x| x.path())
            .collect::<Vec<PathBuf>>()
            .hash(&mut s);
        let hash = s.finish();

        if lock == format!("{:x}", hash) {
            return Some(true);
        }

        Some(false)
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
    sort_photos(&mut files);

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

/// Magic stuff to sort files by name or number
/// Only does by num if the name is formatted like this
/// <NAME><NUM>.<EXT>
/// EX: dog-10.png
pub fn sort_photos(photos: &mut [DirEntry]) {
    photos.sort_by(|x, y| {
        let x = x.path();
        let y = y.path();
        let x = x.to_str().unwrap();
        let y = y.to_str().unwrap();

        if END_NUM_REGEX.is_match(x) && END_NUM_REGEX.is_match(y) {
            let x_find = END_NUM_REGEX.find(x);
            let y_find = END_NUM_REGEX.find(y);

            if x_find.is_none() || y_find.is_none() {
                return x.cmp(y);
            }

            let x_find = x_find.unwrap();
            let y_find = y_find.unwrap();

            if x_find.end() != x.len() || y_find.end() != y.len() {
                return x.cmp(y);
            }

            let x_num: u32 = x_find.as_str().split('.').next().unwrap().parse().unwrap();
            let y_num: u32 = y_find.as_str().split('.').next().unwrap().parse().unwrap();

            return x_num.cmp(&y_num);
        }

        x.cmp(y)
    });
}
