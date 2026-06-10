use anyhow::Result;
use ratatui::Terminal;
use secrecy::SecretString;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::db::client::DbClient;
use crate::db::client::DbCommand;
use crate::events::{AppEvent, EventBridge, spawn_auto_save, spawn_ticker};
use crate::lua::LuaRuntime;
use crate::state::{AppState, load_session_from_disk, save_session_to_disk};
use crate::ui;

pub struct App {
    state: AppState,
    bridge: EventBridge,
    terminal: Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
}

impl App {
    pub fn new(terminal: Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>) -> Self {
        let bridge = EventBridge::new();

        let (db_tx, db_rx) = mpsc::unbounded_channel();
        let mut state = AppState::new();
        state.db_tx = Some(db_tx.clone());

        if let Ok(Some(session)) = load_session_from_disk() {
            let saved_sources = session.saved_sources.clone();
            let active_connection = session.active_connection.clone();
            state.apply_session_data(session);

            for profile_name in &saved_sources {
                if let Some(profile) =
                    state.config.connections.iter().find(|p| p.name == *profile_name)
                {
                    let password = profile.get_password().unwrap_or_default();
                    let dsn = build_dsn_from_profile(profile, &password);
                    let engine_type = match profile.db_type.as_str() {
                        "mysql" => crate::db::backend::EngineType::Mysql,
                        "sqlite" => crate::db::backend::EngineType::Sqlite,
                        _ => crate::db::backend::EngineType::Postgres,
                    };
                    let masked = crate::state::mask_raw_dsn(&dsn);
                    state.connections.push(crate::state::ConnectionEntry {
                        name: profile_name.clone(),
                        engine_type,
                        status: crate::state::ConnectionStatus::Connecting {
                            dsn: dsn.clone(),
                            masked: masked.clone(),
                        },
                        masked_dsn: masked,
                    });
                    let _ = db_tx.send(DbCommand::Connect {
                        connection_name: profile_name.clone(),
                        dsn: SecretString::from(dsn),
                        engine_type,
                    });
                }
            }

            if let Some(ref active) = active_connection {
                state.active_connection = Some(active.clone());
            }
        }

        match LuaRuntime::new(Some(db_tx.clone())) {
            Ok(runtime) => {
                runtime.load_init();
                runtime.load_plugins();
                let resolved_theme = runtime.resolve_theme(state.theme.clone());
                state.theme = resolved_theme;
                state.lua_runtime = Some(runtime);
            },
            Err(e) => {
                error!("Failed to initialize Lua runtime: {}", e);
            },
        }

        let db_client = DbClient::new(db_rx, bridge.tx.clone());
        tokio::spawn(async move {
            let mut client = db_client;
            client.run().await;
        });

        Self { state, bridge, terminal }
    }

    pub async fn run(&mut self) -> Result<()> {
        let tx = self.bridge.tx.clone();
        let _poller = EventBridge::spawn_poller(tx.clone());
        let _ticker = spawn_ticker(tx.clone());
        let _auto_save = spawn_auto_save(tx);

        info!("Application started");

        loop {
            self.terminal.draw(|f| ui::render(f, &self.state))?;

            match self.bridge.rx.recv().await {
                Some(AppEvent::Key(key)) => {
                    self.state.handle_key(key);
                },
                Some(AppEvent::DbEvent(e)) => {
                    self.state.apply_db_event(&e);
                },
                Some(AppEvent::Tick) => {
                    self.state.tick();
                },
                Some(AppEvent::AutoSave) => {
                    let session_data = self.state.to_session_data();
                    std::thread::spawn(move || {
                        if let Err(e) = save_session_to_disk(&session_data) {
                            tracing::error!("Auto-save failed: {}", e);
                        }
                    });
                },
                Some(AppEvent::Resize(_w, _h)) => {},
                Some(AppEvent::Error(e)) => {
                    tracing::error!("Event error: {:?}", e);
                    self.state.should_quit = true;
                },
                None => break,
            }

            if self.state.should_quit {
                break;
            }
        }

        let session_data = self.state.to_session_data();
        if let Err(e) = save_session_to_disk(&session_data) {
            tracing::error!("Failed to save session: {}", e);
        }

        info!("Application shutting down");
        Ok(())
    }
}

fn build_dsn_from_profile(profile: &crate::config::ConnectionProfile, password: &str) -> String {
    match profile.db_type.as_str() {
        "mysql" => {
            let mut dsn = String::from("mysql://");
            if !profile.user.is_empty() {
                dsn.push_str(&profile.user);
                if !password.is_empty() {
                    dsn.push(':');
                    dsn.push_str(password);
                }
                dsn.push('@');
            }
            dsn.push_str(&profile.host);
            if profile.port > 0 {
                dsn.push(':');
                dsn.push_str(&profile.port.to_string());
            }
            if !profile.database.is_empty() {
                dsn.push('/');
                dsn.push_str(&profile.database);
            }
            dsn
        },
        "sqlite" => format!("sqlite://{}", profile.host),
        _ => {
            let mut dsn = String::from("postgresql://");
            if !profile.user.is_empty() {
                dsn.push_str(&profile.user);
                if !password.is_empty() {
                    dsn.push(':');
                    dsn.push_str(password);
                }
                dsn.push('@');
            }
            dsn.push_str(&profile.host);
            if profile.port > 0 {
                dsn.push(':');
                dsn.push_str(&profile.port.to_string());
            }
            if !profile.database.is_empty() {
                dsn.push('/');
                dsn.push_str(&profile.database);
            }
            dsn
        },
    }
}
