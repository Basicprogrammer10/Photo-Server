use std::time::Instant;

use afire::{Method, Request, Response, Server};

use crate::ALBUMS;
use crate::LOGGING;

mod cover;
mod page;
mod photo;
mod photos;
mod preview;
mod thumb;

const TIME_UNITS: &[&str] = &["μs", "ms", "s"];

pub fn attach(server: &mut Server) {
    server.middleware(Box::new(|req| {
        let start = Instant::now();

        let res = match middleware(req) {
            Some(i) => i,
            None => Some(
                Response::new()
                    .status(500)
                    .text("Internal Server Error\nSorwy..."),
            ),
        };

        let end = start.elapsed().as_micros();

        if unsafe { LOGGING } {
            println!(
                "\x1b[35m({}) \x1b[36m[{}] \x1b[31m{} {}",
                if res.is_some() {
                    time_unit(end)
                }
                /* im sorry */
                else {
                    "XXXXX".to_string()
                },
                req.address.split(':').next().unwrap_or(&req.address),
                req.method,
                path_str(req.path.clone()).unwrap_or_else(|| req.path.clone())
            );
        }

        res
    }));
}

fn middleware(req: &Request) -> Option<Option<Response>> {
    if req.method != Method::GET {
        return None;
    }

    let path = req.path.to_lowercase();

    for i in unsafe { ALBUMS.clone() }? {
        if path == i.host_path {
            return Some(page::page(i));
        }

        if path == format!("{}/photos", i.host_path) {
            return Some(photos::photos(i));
        }

        if path == format!("{}/cover", i.host_path) {
            return Some(cover::cover(i));
        }

        if path.starts_with(&format!("{}/photo/", i.host_path)) {
            return Some(photo::photo(i, req));
        }

        if path.starts_with(&format!("{}/preview/", i.host_path)) {
            return Some(preview::photo(i, req));
        }

        if path.starts_with(&format!("{}/thumb/", i.host_path)) {
            return Some(thumb::photo(i, req));
        }
    }

    Some(None)
}

fn image_type(ending: &str) -> &str {
    match ending.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        _ => "",
    }
}

fn time_unit(time: u128) -> String {
    let mut time = time;
    for i in TIME_UNITS {
        if time < 1000 {
            return format!("{:0>3}{: <2}", time, i);
        }
        time /= 1000;
    }

    return format!("{:0>3}{: <2}", time, TIME_UNITS.last().unwrap());
}

fn path_str(path: String) -> Option<String> {
    let mut new_path = String::new();
    let path = path.replace("%20", " ");
    let mut parts = path.split('/').skip(1);

    new_path.push_str("\x1b[32m/");
    new_path.push_str(parts.next()?);

    for i in parts {
        new_path.push_str("\x1b[");
        new_path.push_str(
            &match i {
                "photos" => 34,
                "cover" => 34,
                "photo" => 34,
                "thumb" => 34,
                _ => 33,
            }
            .to_string(),
        );
        new_path.push_str("m/");
        new_path.push_str(i);
    }

    new_path.push_str("\x1b[0m");

    Some(new_path)
}
