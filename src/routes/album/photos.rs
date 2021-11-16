use std::fs;

use afire::{Header, Response};

use crate::album::Album;
use crate::IMAGE_FORMATS;

/// Get all album Images
///
/// Format that as a JSON response
pub fn photos(i: Album) -> Option<Response> {
    let mut files = fs::read_dir(i.path.join(i.images_path))
        .ok()?
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();

    files.sort_by_key(|x| x.file_name());

    let mut images = String::new();

    for file in files {
        let file_name = file.file_name().into_string().ok()?;

        if IMAGE_FORMATS.contains(&file_name.split('.').last()?.to_lowercase().as_str()) {
            images.push_str(&format!(r#""{}","#, file_name));
        }
    }

    return Some(
        Response::new()
            .text(format!(r#"[{}]"#, &images[..images.len() - 1]))
            .header(Header::new("Content-Type", "application/json")),
    );
}
