use afire::Server;

mod album;
mod index;

pub fn attach(server: &mut Server) {
    album::attach(server);
    index::attach(server);
}
