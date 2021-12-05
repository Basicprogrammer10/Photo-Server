use std::fs;

use afire::{Header, Request, Response};

use super::image_type;
use crate::album::Album;

/// Get an Image Thumbnail
pub fn photo(i: Album, req: &Request) -> Option<Response> {
    let image = req.path.splitn(2, "/thumb/").last()?.replace("%20", " ");
    let cache = i.path.join(".thumbs");

    if !cache.exists() {
        println!("[*] Genarateing Thumbnails for `{}`", i.name);
        i.gen_thumbs()?;
        println!("[*] Done!");
    }

    let path = cache.join(image);
    let image_data = match fs::read(path.clone()) {
        Ok(i) => i,
        Err(_) => return Some(Response::new().status(400).text("No Image Found")),
    };

    let file_type = image_type(path.as_path().extension()?.to_str()?);

    Some(
        Response::new()
            .bytes(image_data)
            .header(Header::new("Content-Type", file_type)),
    )
}
