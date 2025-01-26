use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};

use crate::{
    config::{Config, ConfigService},
    error::Result,
    model::heartbeat,
};

pub async fn handler(
    State(db): State<DatabaseConnection>,
    Path(token): Path<String>,
) -> Result<StatusCode> {
    let cfg = Config::get_instance();

    // Get service name by token
    let service = cfg.services.iter().find(|s| {
        if let ConfigService::Http { token: target, .. } = s {
            token == *target
        } else {
            false
        }
    });
    if service.is_none() {
        return Ok(StatusCode::NOT_FOUND);
    }
    let name = service.unwrap().get_name();

    // Create heartbeat
    let heartbeat = heartbeat::ActiveModel {
        name: ActiveValue::Set(format!("http:{}", name)),
        ..Default::default()
    };
    heartbeat::Entity::insert(heartbeat).exec(&db).await?;

    // Success
    Ok(StatusCode::NO_CONTENT)
}
