#[macro_use]
extern crate lazy_static;
use afire::Server;
use simple_config_parser::Config;

mod album;
mod cache;
mod image;
mod routes;
mod serve_static;
mod template;
use album::Album;
use template::Template;

use crate::album::ProcessType;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const IMAGE_FORMATS: &[&str] = &["png", "jpg", "jpeg"];

pub static mut ALBUMS: Option<Vec<Album>> = None;
pub static mut LOGGING: bool = false;

fn main() {
    println!("Starting ImgServer V{}\n", VERSION);

    let cfg = Config::new().file("data/config/config.cfg").unwrap();
    let host = cfg.get_str("host").unwrap();
    let port = cfg.get::<u16>("port").unwrap();
    let album_path = cfg.get_str("album_path").unwrap();
    let logging = cfg.get::<bool>("logging").unwrap();
    unsafe { LOGGING = logging }

    let albums = match album::load_albums(album_path) {
        Some(i) => i,
        None => return println!("[-] Error loading Albums..."),
    };

    println!("[*] Loaded {} Albums", albums.len());
    for i in 0..albums.len() {
        for j in ProcessType::all() {
            let updates = albums[i].check(j);
            if updates.is_none() {
                continue;
            }

            println!(" ├── ! Generating {} for Album {}", j, albums[i].name);
            albums[i].gen_cache(updates, j).unwrap();
        }

        if i < albums.len() - 1 {
            println!(" ├── {}: {}", albums[i].name, &albums[i].host_path);
            continue;
        }
        println!(" └── {}: {}", albums[i].name, &albums[i].host_path);
    }

    unsafe { ALBUMS = Some(albums) };

    let mut server = Server::new(host, port);

    serve_static::attach(&mut server);
    routes::attach(&mut server);

    println!("\nStarting Server {}:{}", server.ip, server.port);
    server.start_threaded(16).unwrap();
}
