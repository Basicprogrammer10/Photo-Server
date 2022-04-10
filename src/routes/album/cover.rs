use std::fs;

use afire::Response;

use super::image_type;
use crate::album::Album;

/// Get all album cover Image
pub fn cover(i: Album) -> Option<Response> {
    let path = i.path.join(i.cover_path.clone());
    if !path.exists() {}
    let image = fs::read(path).ok()?;

    return Some(Response::new().bytes(image).header(
        "Content-Type",
        image_type(i.cover_path.extension()?.to_str()?),
    ));
}
