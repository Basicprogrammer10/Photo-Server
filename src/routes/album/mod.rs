use std::fs;

use afire::{Header, Method, Request, Response, Server};

use crate::Template;
use crate::ALBUMS;
use crate::IMAGE_FORMATS;

// TODO: Cleanup this mess
pub fn attach(server: &mut Server) {
    server.middleware(Box::new(|req| match middleware(req) {
        Some(i) => i,
        None => Some(Response::new().text("Internal Server Error\nSorwy...")),
    }));
}

fn middleware(req: &Request) -> Option<Option<Response>> {
    if req.method != Method::GET {
        return None;
    }

    let path = req.path.to_lowercase();

    for i in unsafe { ALBUMS.clone() }? {
        if path == i.host_path {
            let resp = fs::read_to_string("data/template/album.html").ok()?;
            let readme = fs::read_to_string(i.path.join(i.readme_path)).ok()?;

            let resp = Template::new(resp)
                .template("NAME", i.name)
                .template("COVER", format!("{}/cover", i.host_path))
                .template("README", markdown::to_html(&readme))
                .build();

            return Some(Some(
                Response::new()
                    .text(resp)
                    .header(Header::new("Content-Type", "text/html")),
            ));
        }

        if path == format!("{}/photos", i.host_path) {
            let files = fs::read_dir(i.path.join(i.images_path)).ok()?;
            let mut images = String::new();

            for file in files {
                let file = file.ok()?;
                let file_name = file.file_name().into_string().ok()?;

                if IMAGE_FORMATS.contains(&file_name.split('.').last()?.to_lowercase().as_str()) {
                    images.push_str(&format!(r#""{}","#, file_name));
                }
            }

            return Some(Some(
                Response::new()
                    .text(format!(r#"[{}]"#, &images[..images.len() - 1]))
                    .header(Header::new("Content-Type", "application/json")),
            ));
        }

        if path == format!("{}/cover", i.host_path) {
            let image = fs::read(i.path.join(i.cover_path.clone())).ok()?;

            return Some(Some(Response::new().bytes(image).header(Header::new(
                "Content-Type",
                image_type(i.cover_path.extension()?.to_str()?),
            ))));
        }

        if path.starts_with(&format!("{}/photo/", i.host_path)) {
            let image = req.path.splitn(2, "/photo/").last()?.replace("%20", " ");
            let path = i.path.join(i.images_path).join(image);
            let image_data = match fs::read(path.clone()) {
                Ok(i) => i,
                Err(_) => return Some(Some(Response::new().status(400).text("No Image Found"))),
            };

            let file_type = image_type(path.as_path().extension()?.to_str()?);

            return Some(Some(
                Response::new()
                    .bytes(image_data)
                    .header(Header::new("Content-Type", file_type)),
            ));
        }
    }

    Some(None)
}

fn image_type(ending: &str) -> &str {
    match ending {
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        _ => "",
    }
}
