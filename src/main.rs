use std::fs;

use afire::{Header, Method, Response, Server};

mod album;
mod markdown;
mod serve_static;
mod template;
use album::Album;
use template::Template;

pub const VERSION: &str = "*0.0.0";

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

    server.route(Method::GET, "/", |req| {
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
                    .template("README", markdown::render(readme))
                    .build();

                return Some(
                    Response::new()
                        .text(resp)
                        .header(Header::new("Content-Type", "text/html")),
                );
            }
        }

        None
    }));

    server.start().unwrap();
}
