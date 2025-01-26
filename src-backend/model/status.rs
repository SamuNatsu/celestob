use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue};

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "status")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,

    #[sea_orm(indexed)]
    pub timestamp: String,

    pub name: String,
    pub count: i32,
}

#[derive(Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            uuid: ActiveValue::Set(Uuid::new_v4()),
            timestamp: ActiveValue::Set(Utc::now().format("%Y-%m-%dT%H:00:00%:z").to_string()),
            ..ActiveModelTrait::default()
        }
    }
}
