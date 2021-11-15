use std::fs;

use afire::{Header, Method, Response, Server};
use markdown;

mod album;
// mod markdown;
mod serve_static;
mod template;
use album::Album;
use template::Template;

pub const VERSION: &str = "0.0.0*";
pub const IMAGE_FORMATS: &[&str] = &["png", "jpg", "jpeg"];

pub static mut ALBUMS: Option<Vec<Album>> = None;

fn main() {
    println!("Starting ImgServer V{}\n", VERSION);

    let albums = match album::load_albums("data/albums") {
        Some(i) => i,
        None => return println!("[-] Error loading Albums..."),
    };

    println!("[*] Loaded {} Albums", albums.len());
    for i in 0..albums.len() {
        if i < albums.len() - 1 {
            println!(" ├── {}: {}", albums[i].name, &albums[i].host_path);
            continue;
        }
        println!(" └── {}: {}", albums[i].name, &albums[i].host_path);
    }

    unsafe { ALBUMS = Some(albums) };

    let mut server = Server::new("localhost", 3030);

    serve_static::attach(&mut server);

    server.route(Method::GET, "/", |_req| {
        let resp = fs::read_to_string("data/template/index.html").unwrap();

        let mut albums = String::new();
        for i in unsafe { ALBUMS.clone() }.unwrap() {
            albums.push_str(&format!(
                r#"<li><a href="{}">{}</a></li>"#,
                i.host_path, i.name
            ));
        }

        let resp = Template::new(resp)
            .template("ALBUMS", format!("<ul>{}</ul>", albums))
            .build();

        return Response::new()
            .text(resp)
            .header(Header::new("Content-Type", "text/html"));
    });

    server.middleware(Box::new(|req| {
        if req.method != Method::GET {
            return None;
        }

        for i in unsafe { ALBUMS.clone() }.unwrap() {
            if req.path == i.host_path {
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

            if req.path == format!("{}/photos", i.host_path) {
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

            if req.path == format!("{}/cover", i.host_path) {
                let image = fs::read(i.path.join(i.cover_path.clone())).unwrap();

                return Some(Response::new().bytes(image).header(Header::new(
                    "Content-Type",
                    image_type(i.cover_path.extension().unwrap().to_str().unwrap()),
                )));
            }

            if req.path.starts_with(&format!("{}/photo/", i.host_path)) {
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

    server.start().unwrap();
}

fn image_type(ending: &str) -> &str {
    match ending {
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        _ => "",
    }
}
