use std::fs;

use afire::{Method, Response, Server};

use crate::Template;
use crate::ALBUMS;

pub fn attach(server: &mut Server) {
    server.route(Method::GET, "/", |_req| {
        let resp = fs::read_to_string("data/template/index.html").unwrap();

        let mut albums = String::new();
        for i in unsafe { ALBUMS.clone() }.unwrap() {
            albums.push_str(&format!(
                r#"<li><a href="{}">{} ({})</a></li>"#,
                i.host_path, i.name, i.images
            ));
        }

        let resp = Template::new(resp)
            .template("ALBUMS", format!("<ul>{}</ul>", albums))
            .build();

        Response::new()
            .text(resp)
            .header("Content-Type", "text/html")
    });
}
