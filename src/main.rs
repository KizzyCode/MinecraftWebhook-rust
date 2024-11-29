#![doc = include_str!("../README.md")]
// Clippy lints
#![warn(clippy::large_stack_arrays)]
#![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::panic)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::allow_attributes_without_reason)]
#![warn(clippy::cognitive_complexity)]

mod config;
mod error;
mod minecraft;
mod webui;

use crate::{config::Config, error::Error};
use ehttpd::{
    http::{Request, Response, ResponseExt},
    Server,
};
use std::{process, str, sync::Arc};

fn route(request: Request, config: &Arc<Config>) -> Response {
    // Routing
    match (request.method.as_ref(), request.target.as_ref()) {
        (b"POST", endpoint) if endpoint.starts_with(b"/api/") => {
            // Propagate the response to the minecraft endpoint
            minecraft::webhook(&request, config)
        }
        (b"GET", b"/") => {
            // Serve the web-UI site
            webui::site(&request)
        }
        _ => {
            // Log invalid target and return 404
            let target_str = str::from_utf8(&request.target).unwrap_or("<non UTF-8>");
            eprintln!("Invalid request target: {target_str}");

            // Create a 404 response
            let mut response: Response = ResponseExt::new_404_notfound();
            response.set_content_length(0);
            response
        }
    }
}

pub fn main() {
    /// The fallible main function code
    fn fallible() -> Result<(), Error> {
        // Setup periodical database refresh and load config
        let config = Config::load()?;

        // Initialize the server
        let config_ = Arc::new(config.clone());
        let server: Server<_> = Server::new(config.server.connection_limit, move |source, sink| {
            let config = config_.clone();
            ehttpd::reqresp(source, sink, move |request| route(request, &config))
        });

        // Start the server
        server.accept(&config.server.address)?;
        unreachable!("`server.accept` can never exit gracefully")
    }

    // Execute the fallible code and pretty print any error
    if let Err(e) = fallible() {
        // Print error and backtrace
        eprintln!("Fatal error: {e}");
        if e.has_backtrace() {
            eprintln!("{}", e.backtrace);
        }

        // Exit with abnormal status code
        process::exit(1);
    }
}
