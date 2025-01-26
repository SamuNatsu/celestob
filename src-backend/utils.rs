use anyhow::Result;
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, Schema};

/// Initialize database table
pub async fn init_table<E>(db: &DatabaseConnection, entity: E) -> Result<()>
where
    E: EntityTrait,
{
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    // Create table
    let mut stmt = schema.create_table_from_entity(entity);
    stmt.if_not_exists();
    db.execute(backend.build(&stmt)).await?;

    // Create indices
    let mut stmts = schema.create_index_from_entity(entity);
    for stmt in stmts.iter_mut() {
        stmt.if_not_exists();
        db.execute(backend.build(stmt)).await?;
    }

    // Success
    Ok(())
}
