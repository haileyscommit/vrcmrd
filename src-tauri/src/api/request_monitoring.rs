use reqwest_middleware::{Middleware, Next};
use reqwest::{Request, Response};
use tauri::http::{HeaderMap, Extensions};
use std::fmt::Write;

pub struct LoggingMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        ext: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        // Log request
        log_request(&req);

        // Execute request
        let res = next.run(req, ext).await?;

        // Log response
        log_response(&res).await;

        Ok(res)
    }
}

fn log_request(req: &Request) {
    let mut output = String::new();
    let _ = writeln!(output, "{} {} HTTP/1.1", req.method(), req.url());
    log_headers(req.headers(), &mut output);
    println!("{}", output);
}

async fn log_response(res: &Response) {
    let mut output = String::new();
    let status = res.status();
    let _ = writeln!(output, "HTTP/1.1 {} {}", status.as_u16(), status.canonical_reason().unwrap_or(""));
    log_headers(res.headers(), &mut output);
    println!("{}", output);
}

fn log_headers(headers: &HeaderMap, output: &mut String) {
    for (name, value) in headers.iter() {
        let _ = writeln!(output, "{}: {}", name, value.to_str().unwrap_or("<invalid>"));
    }
    let _ = writeln!(output);
}