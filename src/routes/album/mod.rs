use std::fs;

use afire::{Header, Method, Response, Server};

use crate::Template;
use crate::ALBUMS;
use crate::IMAGE_FORMATS;

// TODO: Cleanup this mess
pub fn attach(server: &mut Server) {
    server.middleware(Box::new(|req| {
        if req.method != Method::GET {
            return None;
        }

        let path = req.path.to_lowercase();

        for i in unsafe { ALBUMS.clone() }.unwrap() {
            if path == i.host_path {
                let resp = fs::read_to_string("data/template/album.html").unwrap();
                let readme = fs::read_to_string(i.path.join(i.readme_path)).unwrap();

                let resp = Template::new(resp)
                    .template("NAME", i.name)
                    .template("COVER", format!("{}/cover", i.host_path))
                    .template("README", markdown::to_html(&readme))
                    .build();

                return Some(
                    Response::new()
                        .text(resp)
                        .header(Header::new("Content-Type", "text/html")),
                );
            }

            if path == format!("{}/photos", i.host_path) {
                let files = fs::read_dir(i.path.join(i.images_path)).unwrap();
                let mut images = String::new();

                for file in files {
                    let file = file.unwrap();
                    let file_name = file.file_name().into_string().unwrap();

                    if IMAGE_FORMATS.contains(&file_name.split('.').last().unwrap()) {
                        images.push_str(&format!(r#""{}","#, file_name));
                    }
                }

                return Some(
                    Response::new()
                        .text(format!(r#"[{}]"#, &images[..images.len() - 1]))
                        .header(Header::new("Content-Type", "application/json")),
                );
            }

            if path == format!("{}/cover", i.host_path) {
                let image = fs::read(i.path.join(i.cover_path.clone())).unwrap();

                return Some(Response::new().bytes(image).header(Header::new(
                    "Content-Type",
                    image_type(i.cover_path.extension().unwrap().to_str().unwrap()),
                )));
            }

            if path.starts_with(&format!("{}/photo/", i.host_path)) {
                let image = req.path.splitn(2, "/photo/").last().unwrap();
                let path = i.path.join(i.images_path).join(image);
                let image_data = match fs::read(path.clone()) {
                    Ok(i) => i,
                    Err(_) => return Some(Response::new().status(400).text("No Image Found")),
                };

                let file_type = image_type(path.as_path().extension().unwrap().to_str().unwrap());

                return Some(
                    Response::new()
                        .bytes(image_data)
                        .header(Header::new("Content-Type", file_type)),
                );
            }
        }

        None
    }));
}

fn image_type(ending: &str) -> &str {
    match ending {
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        _ => "",
    }
}
