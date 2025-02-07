use std::collections::HashMap;

use axum::{extract::State, Json};
use chrono::{DateTime, Duration, Timelike, Utc};
use itertools::Itertools;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;

use crate::{
    config::{Config, ConfigService},
    error::Result,
    model::status,
};

#[derive(Serialize)]
pub struct Response {
    pub pivot: String,
    pub services: Vec<Service>,
}

#[derive(Serialize)]
pub struct Service {
    pub name: String,
    pub desc: String,
    pub stat: Vec<Option<i32>>,
}

pub async fn handler(State(db): State<DatabaseConnection>) -> Result<Json<Response>> {
    let cfg = Config::get_instance();

    // Fetch status
    let result = status::Entity::find()
        .filter(status::Column::Timestamp.gte(Utc::now() - Duration::days(2)))
        .all(&db)
        .await?
        .into_iter()
        .into_group_map_by(|v| v.name.clone());

    // Get pivot
    let pivot = Utc::now()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    // Compute stats
    fn compute_stat(v: &Vec<status::Model>, pivot: &DateTime<Utc>) -> Vec<Option<i32>> {
        let tmp = v
            .clone()
            .into_iter()
            .map(|v| (v.timestamp, v.count))
            .collect::<HashMap<_, _>>();

        let mut tmp = (0..48)
            .into_iter()
            .map(|i| {
                tmp.get(
                    &(*pivot - Duration::hours(i))
                        .format("%Y-%m-%dT%H:00:00%:z")
                        .to_string(),
                )
                .map(|v| v.clone())
            })
            .collect::<Vec<_>>();
        while tmp.len() > 0 && tmp.last().unwrap().is_none() {
            tmp.pop();
        }

        tmp
    }

    // Compute return services
    let services = cfg
        .services
        .iter()
        .map(|s| match s {
            ConfigService::Http {
                name, description, ..
            } => Service {
                name: format!("http:{}", name),
                desc: description.clone(),
                stat: result
                    .get(&format!("http:{}", name))
                    .map_or(vec![], |v| compute_stat(v, &pivot)),
            },
            ConfigService::Docker {
                name, description, ..
            } => Service {
                name: format!("docker:{}", name),
                desc: description.clone(),
                stat: result
                    .get(&format!("docker:{}", name))
                    .map_or(vec![], |v| compute_stat(v, &pivot)),
            },
        })
        .collect::<Vec<_>>();

    // Success
    Ok(Json(Response {
        pivot: pivot.to_rfc3339(),
        services,
    }))
}
