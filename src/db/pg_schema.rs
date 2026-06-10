use crate::events::ColumnInfo;
use crate::events::TableInfo;
use crate::explorer::SchemaNode;
use sqlx::{PgPool, Row};
use std::collections::BTreeMap;
use tracing::error;

pub async fn load_pg_schema(pool: &PgPool) -> Result<Vec<SchemaNode>, String> {
    let rows = sqlx::query(
        "SELECT table_schema, table_name, table_type
         FROM information_schema.tables
         WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
         ORDER BY table_schema, table_name",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Schema query: {e}"))?;

    let mut schemas: BTreeMap<String, Vec<SchemaNode>> = BTreeMap::new();

    for row in &rows {
        let schema_name: String = row.get("table_schema");
        let table_name: String = row.get("table_name");
        let table_type: String = row.get("table_type");

        let node = match table_type.as_str() {
            "VIEW" => SchemaNode::View { schema: schema_name.clone(), name: table_name },
            _ => SchemaNode::Table {
                schema: schema_name.clone(),
                name: table_name,
                expanded: false,
                loaded: false,
                children: Vec::new(),
            },
        };

        schemas.entry(schema_name).or_default().push(node);
    }

    let tree: Vec<SchemaNode> = schemas
        .into_iter()
        .map(|(name, children)| SchemaNode::Schema { name, expanded: false, children })
        .collect();

    Ok(tree)
}

pub async fn load_pg_columns(
    pool: &PgPool,
    schema: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>, String> {
    let rows = sqlx::query(
        "SELECT c.column_name, c.data_type, c.is_nullable,
                CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key
         FROM information_schema.columns c
         LEFT JOIN (
             SELECT ku.column_name
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage ku
                 ON tc.constraint_name = ku.constraint_name
                 AND tc.table_schema = ku.table_schema
             WHERE tc.constraint_type = 'PRIMARY KEY'
                 AND tc.table_schema = $1
                 AND tc.table_name = $2
         ) pk ON c.column_name = pk.column_name
         WHERE c.table_schema = $1 AND c.table_name = $2
         ORDER BY c.ordinal_position",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("LoadColumns query failed: {e}");
        format!("{e}")
    })?;

    let columns: Vec<ColumnInfo> = rows
        .iter()
        .map(|row| {
            let name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            let nullable_str: String = row.get("is_nullable");
            let is_pk: bool = row.get("is_primary_key");
            ColumnInfo { name, data_type, nullable: nullable_str == "YES", is_primary_key: is_pk }
        })
        .collect();

    Ok(columns)
}

pub async fn load_pg_table_info(
    pool: &PgPool,
    schema: &str,
    table: &str,
) -> Result<TableInfo, String> {
    let ddl = match sqlx::query_scalar::<_, String>(
        "SELECT pg_get_tabledef(oid) FROM pg_class c
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = $1 AND c.relname = $2",
    )
    .bind(schema)
    .bind(table)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(d)) => Some(d),
        Ok(None) => None,
        Err(e) => {
            error!("load_pg_table_info DDL query failed: {e}");
            None
        },
    };

    let row_count = match sqlx::query_scalar::<_, i64>(
        "SELECT n_live_tup FROM pg_stat_user_tables
         WHERE schemaname = $1 AND relname = $2",
    )
    .bind(schema)
    .bind(table)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(n)) if n >= 0 => Some(n as u64),
        Ok(_) => None,
        Err(e) => {
            error!("load_pg_table_info row count query failed: {e}");
            None
        },
    };

    let table_size = match sqlx::query_scalar::<_, i64>(
        "SELECT pg_total_relation_size(c.oid) FROM pg_class c
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = $1 AND c.relname = $2",
    )
    .bind(schema)
    .bind(table)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(bytes)) if bytes >= 0 => Some(format_size(bytes as u64)),
        Ok(_) => None,
        Err(e) => {
            error!("load_pg_table_info size query failed: {e}");
            None
        },
    };

    Ok(TableInfo { ddl, row_count, table_size })
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx + 1 < UNITS.len() {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}
