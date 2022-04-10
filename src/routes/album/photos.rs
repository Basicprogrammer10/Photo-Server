use std::fs;
use std::time::Duration;

use afire::Response;
use mut_static::MutStatic;

use crate::album::{sort_photos, Album};
use crate::cache::Cache;
use crate::IMAGE_FORMATS;

lazy_static! {
    static ref CACHE: MutStatic<Cache::<Album, String>> =
        MutStatic::from(Cache::new(Duration::from_secs(60 * 5)));
}

/// Get all album Images
///
/// Format that as a JSON response
pub fn photos(i: Album) -> Option<Response> {
    if let Some(i) = CACHE.read().unwrap().get(i.clone()) {
        return Some(
            Response::new()
                .text(i)
                .header("Content-Type", "application/json")
                .header("X-Cached", "true"),
        );
    };

    let mut files = fs::read_dir(i.path.join(i.clone().images_path))
        .ok()?
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();
    sort_photos(&mut files);

    let mut images = String::new();
    for file in files {
        let file_name = file.file_name().into_string().ok()?;

        if IMAGE_FORMATS.contains(&file_name.split('.').last()?.to_lowercase().as_str()) {
            images.push_str(&format!(r#""{}","#, file_name));
        }
    }

    let text = format!(r#"[{}]"#, &images[..images.len() - 1]);
    CACHE.write().unwrap().update(i, text.clone());

    Some(
        Response::new()
            .text(text)
            .header("Content-Type", "application/json"),
    )
}
