//! The minecraft webhook endpoint

mod rcon;

use crate::config::Config;
use ehttpd::http::{Request, Response, ResponseExt};
use sha2::{Digest, Sha512_256};
use std::{collections::BTreeMap, str, sync::OnceLock};

/// Resolves a webhook command from it's name
fn lookup_any(name: &[u8], config: &Config) -> Option<&'static String> {
    /// The hash secret to perform a blinded lookup
    static SECRET: OnceLock<[u8; 32]> = OnceLock::new();
    let secret = SECRET.get_or_init(|| {
        // Generate a random secret
        let mut secret = [0; 32];
        getrandom::getrandom(&mut secret).expect("failed to create blinding secret");
        secret
    });

    /// The blinded webhook table
    static HOOKS: OnceLock<BTreeMap<[u8; 32], String>> = OnceLock::new();
    let hooks = HOOKS.get_or_init(|| {
        // Create the blinded hook database
        let mut hooks = BTreeMap::new();
        for (name, command) in &config.webhooks.hooks {
            // Hash the dict key with the secret
            let name = Sha512_256::new().chain_update(name).chain_update(secret).finalize();
            hooks.insert(name.into(), command.clone());
        }
        hooks
    });

    // Hash the webhook name and look it up
    let name: [u8; 32] = Sha512_256::new().chain_update(name).chain_update(secret).finalize().into();
    hooks.get(&name)
}

/// Performs a webhook
pub fn webhook(request: &Request, config: &Config) -> Response {
    // Deny non-post requests
    if request.method != b"POST" {
        // Log invalid method and return 405
        let method_str = str::from_utf8(&request.method).unwrap_or("<non UTF-8>");
        eprintln!("Invalid request method for webhook: {method_str}");

        // Handle non-post methods with 405
        let mut response: Response = ResponseExt::new_405_methodnotallowed();
        response.set_content_length(0);
        return response;
    }

    // Lookup webhook command
    let name = request.target.strip_prefix(b"/api/").expect("called endpoint with invalid prefix");
    let Some(command) = lookup_any(name, config) else {
        // Log invalid target and return 404
        let target_str = str::from_utf8(&request.target).unwrap_or("<non UTF-8>");
        eprintln!("Invalid webhook name: {target_str}");
        
        // Return 404
        let mut response: Response = ResponseExt::new_404_notfound();
        response.set_content_length(0);
        return response;
    };

    // Execute RCON command
    match rcon::exec(config, command) {
        Ok(rcon_response) => {
            // Create 200 OK response
            let mut response: Response = ResponseExt::new_200_ok();
            response.set_field("Content-Type", "text/plain");
            response.set_body_data(rcon_response);
            response
        }
        Err(e) => {
            // Log error
            eprintln!("Failed to execute RCON command: {e}");
            if e.has_backtrace() {
                eprintln!("{}", e.backtrace);
            }

            // Create 500 response
            let mut response: Response = ResponseExt::new_500_internalservererror();
            response.set_content_length(0);
            response
        }
    }
}
