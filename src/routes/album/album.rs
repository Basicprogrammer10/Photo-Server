use std::fs;

use afire::{Header, Response};

use crate::album::Album;
use crate::Template;

/// Main page for an album
///
/// This templates in Name, Cover Image and Readme
pub fn album(i: Album) -> Option<Response> {
    let resp = fs::read_to_string("data/template/album.html").ok()?;
    let readme = fs::read_to_string(i.path.join(i.readme_path)).ok()?;

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
