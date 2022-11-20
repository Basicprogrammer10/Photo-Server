use std::fmt::{Display, Formatter};
use std::fs::{self, DirEntry};
use std::hash::Hash;
use std::io;
use std::path::{Path, PathBuf};

use fast_image_resize as fir;
use hashbrown::{HashMap, HashSet};
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

#[derive(Debug)]
pub struct ImageUpdates {
    added: HashSet<PathBuf>,
    removed: HashSet<PathBuf>,
}

#[derive(Debug)]
pub enum AlbumError {
    Config(io::Error),
    ConfigParse(simple_config_parser::ConfigError),
    Io(),
}

#[derive(Clone, Copy)]
pub enum ProcessType {
    Preview,
    Thumbnail,
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

        let name = try_config(&config, "name")?;
        let author = try_config(&config, "author")?;
        let mut host_path = try_config(&config, "host_path")?;
        let cover_path = try_config(&config, "cover_path")?.parse().unwrap();
        let readme_path = try_config(&config, "readme")?.parse().unwrap();
        let images_path = try_config(&config, "image_dir")?.parse().unwrap();

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

    pub fn gen_cache(&self, updates: ImageUpdates, check_type: ProcessType) -> Option<()> {
        let cache = self.path.join(check_type.to_path());
        let image_size = check_type.to_image_size();
        // let mut hash = Vec::new();
        let mut files = fs::read_dir(self.path.join(self.clone().images_path))
            .ok()?
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        sort_photos(&mut files);

        if !cache.exists() {
            fs::create_dir(&cache).ok()?;
        }

        // Remove old files
        // TODO: this is stupid - and you know it
        for i in updates.removed.iter().filter(|x| x.exists()) {
            fs::remove_file(cache.join(i.file_name()?)).ok()?;
        }

        let bar = ProgressBar::new(files.len() as u64);
        bar.set_style(PROGRESS_STYLE.to_owned());

        for file in updates.added {
            let img = image::open(&file).unwrap();
            let dst_image = crate::image::scale_image(
                &img,
                image_size.0,
                image_size.1,
                fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3),
            );

            image::save_buffer(
                cache.join(file.file_name()?),
                dst_image.buffer(),
                dst_image.width().get(),
                dst_image.height().get(),
                img.color(),
            )
            .unwrap();
            bar.inc(1);
        }

        fs::write(cache.join(".lock"), self.recalculate_hash(check_type)).ok()?;

        Some(())
    }

    pub fn check(&self, check_type: ProcessType) -> ImageUpdates {
        let cache = self.path.join(check_type.to_path());
        let lock = match fs::read_to_string(cache.join(".lock")) {
            Ok(i) => i,
            Err(_) => return ImageUpdates::all(&self.path.join(&self.images_path)),
        };

        // Make a hash map to all cached images and thair hashes
        let mut files = HashMap::new();
        for i in lock.lines() {
            let split = i.splitn(2, ':').collect::<Vec<_>>();
            if split.len() != 2 {
                continue;
            }

            files.insert(split[0], split[1]);
        }

        let mut added = HashSet::new();
        for i in fs::read_dir(self.path.join(&self.images_path))
            .unwrap()
            .map(Result::unwrap)
            .filter(|x| x.path().is_file())
            .filter(|x| x.path().file_name().unwrap() != ".lock")
        {
            // Try to remove the file from the hash map
            // If not found or its hash is not its real hash add it to the added hash set
            let filename = i.file_name();
            let filename = filename.to_string_lossy();
            let filename = filename.as_ref();

            // Try to get the hash from the .lock file
            let stored_hash = match files.remove(filename) {
                Some(i) => i,
                None => {
                    added.insert(i.path());
                    continue;
                }
            };

            // Try to read the file
            let raw_image = match fs::read(cache.join(filename)) {
                Ok(i) => i,
                Err(_) => {
                    added.insert(i.path());
                    continue;
                }
            };

            // Hash the file and compare its hash to the stored hash
            let hash = base16ct::lower::encode_string(&md5::compute(raw_image).0);
            if stored_hash != hash {
                added.insert(i.path());
            }
        }

        // Files to remove are the remaining files in the hash map
        // and files in added are to be added / rebuilt
        let removed = files.keys().map(PathBuf::from).collect::<HashSet<_>>();
        ImageUpdates { added, removed }
    }

    fn recalculate_hash(&self, check_type: ProcessType) -> String {
        let mut out = Vec::new();

        for i in fs::read_dir(self.path.join(check_type.to_path()))
            .unwrap()
            .map(Result::unwrap)
            .filter(|x| x.path().is_file())
            .filter(|x| x.path().file_name().unwrap() != ".lock")
        {
            let raw_image = fs::read(i.path()).unwrap();
            let hash = base16ct::lower::encode_string(&md5::compute(raw_image).0);
            out.push(format!("{}:{}", i.file_name().to_string_lossy(), hash));
        }

        out.join("\n")
    }
}

impl ImageUpdates {
    /// Add all files in the image directory to the add field
    fn all<T: AsRef<Path>>(path: T) -> Self {
        let mut added = HashSet::new();

        for i in fs::read_dir(path)
            .unwrap()
            .map(Result::unwrap)
            .filter(|x| x.path().is_file())
        {
            added.insert(i.path());
        }

        Self {
            added,
            removed: HashSet::new(),
        }
    }

    pub fn is_none(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

impl ProcessType {
    fn to_path(self) -> PathBuf {
        match self {
            ProcessType::Thumbnail => PathBuf::from(".thumbs"),
            ProcessType::Preview => PathBuf::from(".previews"),
        }
    }

    fn to_image_size(self) -> (u32, u32) {
        match self {
            ProcessType::Thumbnail => (u32::MAX, THUMBNAIL_SIZE),
            ProcessType::Preview => PREVIEW_SIZE,
        }
    }

    pub fn all() -> [Self; 2] {
        [ProcessType::Thumbnail, ProcessType::Preview]
    }
}

impl Display for ProcessType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessType::Thumbnail => write!(f, "thumbnails"),
            ProcessType::Preview => write!(f, "previews"),
        }
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

fn try_config(config: &Config, key: &str) -> Result<String, AlbumError> {
    match config.get_str(key) {
        Ok(i) => Ok(i),
        Err(e) => Err(AlbumError::ConfigParse(e)),
    }
}
