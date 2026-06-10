use crate::events::{ColumnInfo, ForeignKeyInfo, IndexInfo, KeyInfo, TableDetails, TableInfo};
use crate::explorer::{FolderKind, SchemaNode};
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

    let mut schemas: BTreeMap<String, (Vec<SchemaNode>, Vec<SchemaNode>)> = BTreeMap::new();

    for row in &rows {
        let schema_name: String = row.get("table_schema");
        let table_name: String = row.get("table_name");
        let table_type: String = row.get("table_type");

        match table_type.as_str() {
            "VIEW" => {
                schemas
                    .entry(schema_name.clone())
                    .or_default()
                    .1
                    .push(SchemaNode::View { schema: schema_name.clone(), name: table_name });
            },
            _ => {
                schemas.entry(schema_name.clone()).or_default().0.push(SchemaNode::Table {
                    schema: schema_name.clone(),
                    name: table_name,
                    expanded: false,
                    loaded: false,
                    children: Vec::new(),
                });
            },
        };
    }

    let tree: Vec<SchemaNode> = schemas
        .into_iter()
        .map(|(name, (tables, views))| {
            let mut schema_children = Vec::new();
            if !tables.is_empty() {
                schema_children.push(SchemaNode::ObjectFolder {
                    kind: FolderKind::Tables,
                    expanded: false,
                    loaded: true,
                    children: tables,
                });
            }
            if !views.is_empty() {
                schema_children.push(SchemaNode::ObjectFolder {
                    kind: FolderKind::Views,
                    expanded: false,
                    loaded: true,
                    children: views,
                });
            }
            SchemaNode::Schema { name, expanded: false, children: schema_children }
        })
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

pub async fn load_pg_table_details(
    pool: &PgPool,
    schema: &str,
    table: &str,
) -> Result<TableDetails, String> {
    let columns = load_pg_columns(pool, schema, table).await?;

    let key_rows = sqlx::query(
        "SELECT tc.constraint_name, ku.column_name
         FROM information_schema.table_constraints tc
         JOIN information_schema.key_column_usage ku
             ON tc.constraint_name = ku.constraint_name
             AND tc.table_schema = ku.table_schema
         WHERE tc.constraint_type = 'PRIMARY KEY'
             AND tc.table_schema = $1
             AND tc.table_name = $2
         ORDER BY ku.ordinal_position",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("PK query: {e}"))?;

    let mut keys_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in &key_rows {
        let constraint_name: String = row.get("constraint_name");
        let column_name: String = row.get("column_name");
        keys_map.entry(constraint_name).or_default().push(column_name);
    }
    let keys: Vec<KeyInfo> =
        keys_map.into_iter().map(|(name, columns)| KeyInfo { name, columns }).collect();

    let fk_rows = sqlx::query(
        "SELECT tc.constraint_name, ku.column_name,
                ccu.table_name AS ref_table, ccu.column_name AS ref_column
         FROM information_schema.table_constraints tc
         JOIN information_schema.key_column_usage ku
             ON tc.constraint_name = ku.constraint_name
             AND tc.table_schema = ku.table_schema
         JOIN information_schema.constraint_column_usage ccu
             ON ccu.constraint_name = tc.constraint_name
             AND ccu.table_schema = tc.table_schema
         WHERE tc.constraint_type = 'FOREIGN KEY'
             AND tc.table_schema = $1
             AND tc.table_name = $2
         ORDER BY tc.constraint_name, ku.ordinal_position",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("FK query: {e}"))?;

    let mut fk_map: BTreeMap<String, (Vec<String>, String, Vec<String>)> = BTreeMap::new();
    for row in &fk_rows {
        let name: String = row.get("constraint_name");
        let col: String = row.get("column_name");
        let ref_table: String = row.get("ref_table");
        let ref_col: String = row.get("ref_column");
        let entry = fk_map.entry(name).or_insert_with(|| (Vec::new(), ref_table, Vec::new()));
        entry.0.push(col);
        entry.2.push(ref_col);
    }
    let foreign_keys: Vec<ForeignKeyInfo> = fk_map
        .into_iter()
        .map(|(name, (columns, ref_table, ref_columns))| ForeignKeyInfo {
            name,
            columns,
            ref_table,
            ref_columns,
        })
        .collect();

    let idx_rows = sqlx::query(
        "SELECT i.relname AS index_name,
                ix.indisunique AS is_unique,
                ix.indisprimary AS is_primary,
                array_agg(a.attname ORDER BY array_position(ix.indkey, a.attnum)) AS columns
         FROM pg_index ix
         JOIN pg_class t ON t.oid = ix.indrelid
         JOIN pg_class i ON i.oid = ix.indexrelid
         JOIN pg_namespace n ON n.oid = t.relnamespace
         JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
         WHERE n.nspname = $1 AND t.relname = $2
         GROUP BY i.relname, ix.indisunique, ix.indisprimary
         ORDER BY i.relname",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Index query: {e}"))?;

    let indexes: Vec<IndexInfo> = idx_rows
        .iter()
        .map(|row| {
            let name: String = row.get("index_name");
            let is_unique: bool = row.get("is_unique");
            let is_primary: bool = row.get("is_primary");
            let columns: Vec<String> = row.get("columns");
            IndexInfo { name, columns, is_unique, is_primary }
        })
        .collect();

    Ok(TableDetails { columns, keys, foreign_keys, indexes })
}
