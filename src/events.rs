use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::explorer::SchemaNode;
use crate::result::ColumnMeta;

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct KeyInfo {
    pub name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ForeignKeyInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct TableDetails {
    pub columns: Vec<ColumnInfo>,
    pub keys: Vec<KeyInfo>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
    pub indexes: Vec<IndexInfo>,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub ddl: Option<String>,
    pub row_count: Option<u64>,
    pub table_size: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DbEvent {
    Connected {
        connection_name: String,
    },
    ConnectionFailed {
        connection_name: String,
        message: String,
    },
    Disconnected {
        connection_name: String,
    },
    SchemaLoaded {
        connection_name: String,
        nodes: Vec<SchemaNode>,
    },
    ColumnsLoaded {
        connection_name: String,
        schema: String,
        table: String,
        columns: Vec<ColumnInfo>,
    },
    TableDetailsLoaded {
        connection_name: String,
        schema: String,
        table: String,
        details: TableDetails,
    },
    TableInfoLoaded {
        connection_name: String,
        _schema: String,
        _table: String,
        ddl: Option<String>,
        row_count: Option<u64>,
        table_size: Option<String>,
    },
    QueryStarted {
        connection_name: String,
    },
    ResultColumns {
        connection_name: String,
        columns: Vec<ColumnMeta>,
    },
    QueryRow {
        connection_name: String,
        cells: Vec<String>,
    },
    QueryCompleted {
        connection_name: String,
        _rows_affected: u64,
        _duration_ms: u64,
    },
    QueryError {
        connection_name: String,
        message: String,
    },
    QueryCancelled {
        connection_name: String,
    },
}

pub enum AppEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
    AutoSave,
    DbEvent(DbEvent),
    Error(anyhow::Error),
}

pub struct EventBridge {
    pub rx: mpsc::UnboundedReceiver<AppEvent>,
    pub tx: mpsc::UnboundedSender<AppEvent>,
}

impl EventBridge {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { rx, tx }
    }

    pub fn spawn_poller(tx: mpsc::UnboundedSender<AppEvent>) -> tokio::task::JoinHandle<()> {
        tokio::task::spawn_blocking(move || {
            loop {
                match event::read() {
                    Ok(CrosstermEvent::Key(key)) => {
                        if key.kind != KeyEventKind::Release {
                            let _ = tx.send(AppEvent::Key(key));
                        }
                    },
                    Ok(CrosstermEvent::Resize(w, h)) => {
                        let _ = tx.send(AppEvent::Resize(w, h));
                    },
                    Ok(_) => {},
                    Err(e) => {
                        let _ = tx.send(AppEvent::Error(anyhow::anyhow!(e)));
                        break;
                    },
                }
            }
        })
    }
}

pub fn spawn_ticker(tx: mpsc::UnboundedSender<AppEvent>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            if tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    })
}

pub fn spawn_auto_save(tx: mpsc::UnboundedSender<AppEvent>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if tx.send(AppEvent::AutoSave).is_err() {
                break;
            }
        }
    })
}
