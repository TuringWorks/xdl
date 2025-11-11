//! Simple HTTP server for serving volume visualizations

use anyhow::Result;
use tiny_http::{Response, Server};

pub struct VizServer {
    server: Server,
    port: u16,
}

impl VizServer {
    /// Create a new server on a random available port
    pub fn new() -> Result<Self> {
        let server = Server::http("127.0.0.1:0")
            .map_err(|e| anyhow::anyhow!("Failed to create server: {}", e))?;
        let port = server
            .server_addr()
            .to_ip()
            .ok_or_else(|| anyhow::anyhow!("Could not get server address"))?
            .port();

        Ok(Self { server, port })
    }

    /// Get the port the server is listening on
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Serve HTML content and handle requests
    pub fn serve_html(&self, html: String) {
        println!("Server thread started, listening on port {}", self.port);

        // Serve requests - use blocking recv()
        loop {
            match self.server.recv() {
                Ok(request) => {
                    println!("Received request for: {}", request.url());

                    let response = match request.url() {
                        "/" | "/index.html" => {
                            println!("Serving HTML page ({} bytes)", html.len());
                            Response::from_string(&html).with_header(
                                tiny_http::Header::from_bytes(
                                    &b"Content-Type"[..],
                                    &b"text/html; charset=utf-8"[..],
                                )
                                .unwrap(),
                            )
                        }
                        _ => {
                            println!("404 for: {}", request.url());
                            Response::from_string("404 Not Found").with_status_code(404)
                        }
                    };

                    if let Err(e) = request.respond(response) {
                        eprintln!("Failed to send response: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Server error: {}", e);
                    break;
                }
            }
        }

        println!("Server thread exiting");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = VizServer::new();
        assert!(server.is_ok());
        if let Ok(server) = server {
            assert!(server.port() > 0);
        }
    }

    #[test]
    fn test_server_port() {
        let server = VizServer::new().unwrap();
        let port = server.port();
        // Port should be valid
        assert!(port > 0);
    }
}
