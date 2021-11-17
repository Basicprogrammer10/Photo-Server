use std::fs;

use afire::{Header, Response};
use regex::Regex;

use crate::album::Album;
use crate::IMAGE_FORMATS;

lazy_static! {
    static ref END_NUM_REGEX: Regex = Regex::new(r#"[0-9]+\..+"#).unwrap();
}

/// Get all album Images
///
/// Format that as a JSON response
pub fn photos(i: Album) -> Option<Response> {
    let mut files = fs::read_dir(i.path.join(i.images_path))
        .ok()?
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();

    // Magic stuff to sort files by name or number
    // Only does by num if the name is formatted like this
    // <NAME><NUM>.<EXT>
    // EX: dog-10.png
    files.sort_by(|x, y| {
        let x = x.path();
        let y = y.path();
        let x = x.to_str().unwrap();
        let y = y.to_str().unwrap();

        if END_NUM_REGEX.is_match(x) && END_NUM_REGEX.is_match(y) {
            let x_find = END_NUM_REGEX.find(x);
            let y_find = END_NUM_REGEX.find(y);

            if !x_find.is_some() || !y_find.is_some() {
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
