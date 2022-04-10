use afire::*;
use std::fs;

use crate::Template;
use crate::VERSION;

/// Dir to find files to serve
const DATA_DIR: &str = "./data/static";

pub fn attach(server: &mut afire::Server) {
    server.route(Method::GET, "**", |req| {
        let mut path = format!("{}{}", DATA_DIR, safe_path(req.path.to_owned()));

        // Add Index.html if path ends with /
        if path.ends_with('/') {
            path.push_str("index.html");
        }

        // Also add '/index.html' if path dose not end with a file
        if !path.split('/').last().unwrap_or_default().contains('.') {
            path.push_str("/index.html");
        }

        // Try to read File
        match fs::read(&path) {
            // If its found send it as response
            Ok(content) => Response::new()
                .bytes(content)
                .header("Content-Type", get_type(&path)),

            // If not send 404.html
            Err(_) => Response::new()
                .status(404)
                .text(
                    Template::new(
                        fs::read_to_string("data/template/not_found.html")
                            .unwrap_or_else(|_| "Not Found :/".to_owned()),
                    )
                    .template("VERSION", VERSION)
                    .template("PAGE", req.path)
                    .build(),
                )
                .header("Content-Type", "text/html"),
        }
    });
}

#[inline]
fn safe_path(mut path: String) -> String {
    while path.contains("/..") {
        path = path.replace("/..", "");
    }
    path
}

/// Get the type MMIE content type of a file from its extension
pub fn get_type(path: &str) -> &str {
    match path.split('.').last() {
        Some(ext) => match ext {
            // More Common Types
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "ico" => "image/x-icon",
            "svg" => "image/svg+xml",
            "txt" => "text/plain",
            "bmp" => "image/bmp",
            "tif" => "image/tiff",
            "tiff" => "image/tiff",
            "webp" => "image/webp",
            _ => "application/octet-stream",
        },

        None => "application/octet-stream",
    }
}
