mod api;
pub mod hooks;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::db::client::DbCommand;
use crate::theme::Theme;
use api::TwisterDBA;
use hooks::LuaRegistries;

pub struct LuaRuntime {
    pub lua: mlua::Lua,
    pub registries: Rc<RefCell<LuaRegistries>>,
    #[allow(dead_code)]
    db_tx: Option<mpsc::UnboundedSender<DbCommand>>,
}

impl LuaRuntime {
    pub fn new(db_tx: Option<mpsc::UnboundedSender<DbCommand>>) -> Result<Self> {
        let lua = mlua::Lua::new();

        {
            let instruction_limit: u64 = 10_000_000;
            let hook_interval: u32 = 10_000;
            let counter = Rc::new(RefCell::new(0u64));
            let max_count = instruction_limit / hook_interval as u64;
            lua.set_hook(
                mlua::HookTriggers {
                    every_nth_instruction: Some(hook_interval),
                    ..Default::default()
                },
                move |_lua, _debug| {
                    let mut count = counter.borrow_mut();
                    *count += 1;
                    if *count > max_count {
                        return Err(mlua::Error::runtime(
                            "Lua instruction limit exceeded (10M instructions)",
                        ));
                    }
                    Ok(mlua::VmState::Continue)
                },
            );
        }

        lua.globals()
            .set("os", mlua::Value::Nil)
            .map_err(|e| anyhow::anyhow!("Lua error: {}", e))?;
        lua.globals()
            .set("io", mlua::Value::Nil)
            .map_err(|e| anyhow::anyhow!("Lua error: {}", e))?;

        let registries = Rc::new(RefCell::new(LuaRegistries::default()));

        let twister = TwisterDBA { registries: registries.clone(), db_tx: db_tx.clone() };
        lua.globals()
            .set("twisterDBA", twister)
            .map_err(|e| anyhow::anyhow!("Lua error: {}", e))?;

        Ok(Self { lua, registries, db_tx })
    }

    pub fn load_init(&self) {
        let init_path = config_dir().join("init.lua");
        if !init_path.exists() {
            return;
        }
        match fs::read_to_string(&init_path) {
            Ok(source) => {
                if let Err(e) = self.lua.load(&source).exec() {
                    error!("Lua error in {}: {}", init_path.display(), e);
                } else {
                    info!("Lua: init.lua loaded");
                }
            },
            Err(e) => {
                error!("Failed to read {}: {}", init_path.display(), e);
            },
        }
    }

    pub fn load_plugins(&self) {
        self.load_plugin_dir("lua");
        self.load_plugin_dir("after/plugin");
    }

    fn load_plugin_dir(&self, subdir: &str) {
        let dir = config_dir().join(subdir);
        if !dir.is_dir() {
            return;
        }
        let mut entries: Vec<PathBuf> = match fs::read_dir(&dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|e| e == "lua").unwrap_or(false))
                .collect(),
            Err(e) => {
                error!("Failed to read plugin dir {}: {}", dir.display(), e);
                return;
            },
        };
        entries.sort();

        for path in entries {
            match fs::read_to_string(&path) {
                Ok(source) => {
                    if let Err(e) = self.lua.load(&source).exec() {
                        error!("Lua error in {}: {}", path.display(), e);
                    } else {
                        info!("Lua plugin loaded: {}", path.display());
                    }
                },
                Err(e) => {
                    error!("Failed to read plugin {}: {}", path.display(), e);
                },
            }
        }
    }

    pub fn execute(&self, code: &str) -> Result<String> {
        let value: mlua::Value =
            self.lua.load(code).eval().map_err(|e| anyhow::anyhow!("Lua error: {}", e))?;
        let result = format!("{:?}", value);
        Ok(result)
    }

    pub fn fire_event(&self, event_name: &str, data: mlua::Table) {
        let registries = self.registries.borrow();
        if let Some(hooks) = registries.hooks.get(event_name) {
            for hook in hooks {
                if let Err(e) = hook.call::<()>(data.clone()) {
                    error!("Lua hook error for event '{}': {}", event_name, e);
                }
            }
        }
    }

    pub fn resolve_theme(&self, base: Theme) -> Theme {
        let registries = self.registries.borrow();
        let overrides = &registries.theme_overrides;
        if overrides.is_empty() {
            return base;
        }
        let mut theme = base;
        if let Some(color) = overrides.get("background").and_then(|c| parse_hex_color(c)) {
            theme.bg = color;
        }
        if let Some(color) = overrides.get("editor_bg").and_then(|c| parse_hex_color(c)) {
            theme.editor_bg = color;
        }
        if let Some(color) = overrides.get("keyword").and_then(|c| parse_hex_color(c)) {
            theme.keyword = color;
        }
        if let Some(color) = overrides.get("string").and_then(|c| parse_hex_color(c)) {
            theme.string = color;
        }
        if let Some(color) = overrides.get("number").and_then(|c| parse_hex_color(c)) {
            theme.number = color;
        }
        if let Some(color) = overrides.get("comment").and_then(|c| parse_hex_color(c)) {
            theme.comment = color;
        }
        theme
    }

    pub fn call_extractor(
        &self,
        name: &str,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    ) -> Result<String> {
        let registries = self.registries.borrow();
        let func = registries
            .extractors
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Extractor '{}' not registered", name))?;

        let lua_columns = self.lua.create_table().map_err(|e| anyhow::anyhow!("{}", e))?;
        for (i, col) in columns.iter().enumerate() {
            lua_columns.set(i + 1, col.as_str()).map_err(|e| anyhow::anyhow!("{}", e))?;
        }

        let lua_rows = self.lua.create_table().map_err(|e| anyhow::anyhow!("{}", e))?;
        for (i, row) in rows.iter().enumerate() {
            let lua_row = self.lua.create_table().map_err(|e| anyhow::anyhow!("{}", e))?;
            for (j, cell) in row.iter().enumerate() {
                lua_row.set(j + 1, cell.as_str()).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            lua_rows.set(i + 1, lua_row).map_err(|e| anyhow::anyhow!("{}", e))?;
        }

        let result: String =
            func.call((lua_columns, lua_rows)).map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(result)
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("twisterDBA")
}

fn parse_hex_color(hex: &str) -> Option<ratatui::style::Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(ratatui::style::Color::Rgb(r, g, b))
}
