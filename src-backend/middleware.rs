use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

use crate::config::Config;

pub async fn auth(req: Request, next: Next) -> Result<Response, StatusCode> {
    let cfg = Config::get_instance();
    if let Some(secret) = &cfg.secret {
        let auth = match req.headers().get("Authorization") {
            Some(h) => h,
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        if auth != secret {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    Ok(next.run(req).await)
}
