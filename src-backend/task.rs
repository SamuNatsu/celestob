use std::{collections::HashMap, future::Future};

use anyhow::Result;
use bollard::{container::ListContainersOptions, Docker};
use chrono::{Duration, Timelike, Utc};
use itertools::Itertools;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter,
};
use tokio::time;
use tracing::{debug, error, info};

use crate::{
    config::{Config, ConfigService},
    model::{heartbeat, status},
};

pub fn clean_database(db: &DatabaseConnection) -> impl Future<Output = ()> {
    let task_db = db.clone();

    async fn wrapper(db: &DatabaseConnection) -> Result<()> {
        // Clean outdated heartbeats
        let result = heartbeat::Entity::delete_many()
            .filter(heartbeat::Column::Timestamp.lt(Utc::now() - Duration::days(3)))
            .exec(db)
            .await?;
        debug!("heartbeats cleaned: count={}", result.rows_affected);

        // Clean outdated status
        let result = status::Entity::delete_many()
            .filter(status::Column::Timestamp.lt(Utc::now() - Duration::days(3)))
            .exec(db)
            .await?;
        debug!("status cleaned: count={}", result.rows_affected);

        // Success
        Ok(())
    }

    async move {
        loop {
            // Execute task
            info!("execute task: clean database");
            if let Err(err) = wrapper(&task_db).await {
                error!("task fail: name=clean database, err={}", err);
            } else {
                info!("task success: name=clean database");
            }

            // Next day
            time::sleep(Duration::days(1).to_std().unwrap()).await;
        }
    }
}

pub fn check_containers(db: &DatabaseConnection, docker: &Docker) -> impl Future<Output = ()> {
    let task_db = db.clone();
    let task_docker = docker.clone();

    async fn wrapper(db: &DatabaseConnection, docker: &Docker) -> Result<()> {
        let cfg = Config::get_instance();

        // Get container list
        let result = docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await?
            .into_iter()
            .filter(|r| r.names.is_some() && r.state.is_some());

        // For named containers
        for r in result {
            // Extract name
            let names = r.names.as_ref().unwrap();
            if names.is_empty() {
                continue;
            }
            let name = &names[0];

            // Find service name
            let service = cfg.services.iter().find(|s| {
                if let ConfigService::Docker { container, .. } = s {
                    container == name
                } else {
                    false
                }
            });
            if service.is_none() {
                continue;
            }
            let name = service.unwrap().get_name();

            // Check state
            if r.state.as_ref().unwrap() != "running" {
                debug!(
                    "container down: name={}, state={}",
                    name,
                    r.state.as_ref().unwrap()
                );
                continue;
            }

            // Create heartbeat
            let heartbeat = heartbeat::ActiveModel {
                name: ActiveValue::Set(format!("docker:{}", name)),
                ..Default::default()
            };
            heartbeat::Entity::insert(heartbeat).exec(db).await?;
        }

        // Success
        Ok(())
    }

    async move {
        loop {
            // Crontab: */5 * * * * *
            let now = Utc::now();
            let next_minute = now.minute() / 5 * 5 + 5;
            let next_time = if next_minute >= 60 {
                now.with_minute(next_minute - 60)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap()
                    + Duration::hours(1)
            } else {
                now.with_minute(next_minute)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap()
            };
            let delta = next_time - now;
            time::sleep(delta.to_std().unwrap()).await;

            // Execute task
            info!("execute task: check containers");
            if let Err(err) = wrapper(&task_db, &task_docker).await {
                error!("task fail: name=check containers, err={}", err);
            } else {
                info!("task success: name=check containers");
            }
        }
    }
}

pub fn collect_status(db: &DatabaseConnection) -> impl Future<Output = ()> {
    let task_db = db.clone();

    async fn wrapper(db: &DatabaseConnection) -> Result<()> {
        let cfg = Config::get_instance();
        let now = Utc::now();

        // <service name, [(timestamp, count)]>
        let status = heartbeat::Entity::find()
            .filter(heartbeat::Column::Timestamp.gte(now - Duration::hours(3)))
            .all(db)
            .await?
            .into_iter()
            .into_group_map_by(|v| v.name.clone())
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .into_group_map_by(|v| {
                            v.timestamp.format("%Y-%m-%dT%H:00:00%:z").to_string()
                        })
                        .into_iter()
                        .k_largest_by(3, |a, b| a.0.cmp(&b.0))
                        .map(|(k, v)| (k, v.len()))
                        .collect::<HashMap<_, _>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        // Activate timestamps
        let tms = vec![
            now.format("%Y-%m-%dT%H:00:00%:z").to_string(),
            (now - Duration::hours(1))
                .format("%Y-%m-%dT%H:00:00%:z")
                .to_string(),
        ];

        // Subroutine for insert/update status
        async fn save_status(
            db: &DatabaseConnection,
            name: &String,
            timestamp: &String,
            count: i32,
        ) -> Result<()> {
            // Try fetch status
            let status = status::Entity::find()
                .filter(status::Column::Name.eq(name))
                .filter(status::Column::Timestamp.eq(timestamp))
                .one(db)
                .await?;

            // Insert new or update old
            match status {
                Some(status) => {
                    let mut status = status.into_active_model();
                    status.count = ActiveValue::Set(count);
                    status.update(db).await?;
                }
                None => {
                    let status = status::ActiveModel {
                        timestamp: ActiveValue::Set(timestamp.clone()),
                        name: ActiveValue::Set(name.clone()),
                        count: ActiveValue::Set(count),
                        ..Default::default()
                    };
                    status.insert(db).await?;
                }
            }

            // Success
            Ok(())
        }

        // For each service
        for s in &cfg.services {
            let name = if let ConfigService::Http { .. } = s {
                format!("http:{}", s.get_name())
            } else {
                format!("docker:{}", s.get_name())
            };

            for timestamp in &tms {
                let count = status
                    .get(&name)
                    .map(|status| status.get(timestamp).unwrap_or(&0))
                    .unwrap_or(&0);
                save_status(db, &name, timestamp, *count as i32).await?;
            }
        }

        // Success
        Ok(())
    }

    async move {
        loop {
            // Execute task
            info!("execute task: collect status");
            if let Err(err) = wrapper(&task_db).await {
                error!("task fail: name=collect status, err={}", err);
            } else {
                info!("task success: name=collect status");
            }

            // Wait until next time
            time::sleep(Duration::minutes(4).to_std().unwrap()).await;
        }
    }
}
