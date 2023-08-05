//! The web-UI site

use ehttpd::http::{Request, Response, ResponseExt};

/// The website data
const SITE: &str = include_str!("site.html");

/// Serves the web UI site
pub fn site(_request: &Request) -> Response {
    let mut response: Response = ResponseExt::new_200_ok();
    response.set_body_data(SITE);
    response
}
