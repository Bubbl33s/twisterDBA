use std::cell::RefCell;
use std::rc::Rc;

use tokio::sync::mpsc;
use tracing::{info, warn};

use super::hooks::LuaRegistries;
use crate::db::client::DbCommand;

pub struct TwisterDBA {
    pub registries: Rc<RefCell<LuaRegistries>>,
    pub db_tx: Option<mpsc::UnboundedSender<DbCommand>>,
}

impl mlua::UserData for TwisterDBA {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("setup_theme", |_lua, this, theme_table: mlua::Table| {
            let mut registries = this.registries.borrow_mut();
            for pair in theme_table.pairs::<mlua::Value, mlua::Value>() {
                match pair {
                    Ok((key, value)) => {
                        if let (Some(key_str), Some(val_str)) = (key.as_str(), value.as_str()) {
                            registries
                                .theme_overrides
                                .insert(key_str.to_string(), val_str.to_string());
                        }
                    },
                    Err(e) => {
                        warn!("Invalid theme entry: {}", e);
                    },
                }
            }
            Ok(())
        });

        methods.add_method(
            "set_keymap",
            |_lua, this, (mode, lhs, rhs, _opts): (String, String, mlua::Function, Option<mlua::Table>)| {
                let key = format!("{}|{}", mode, lhs);
                let mut registries = this.registries.borrow_mut();
                registries.keymaps.insert(key, rhs);
                Ok(())
            },
        );

        methods.add_method(
            "register_command",
            |_lua, this, (name, callback): (String, mlua::Function)| {
                let mut registries = this.registries.borrow_mut();
                registries.commands.insert(name, callback);
                Ok(())
            },
        );

        methods.add_method(
            "register_extractor",
            |_lua, this, (name, callback): (String, mlua::Function)| {
                let mut registries = this.registries.borrow_mut();
                registries.extractors.insert(name, callback);
                Ok(())
            },
        );

        methods.add_method(
            "on_event",
            |_lua, this, (event, callback): (String, mlua::Function)| {
                let mut registries = this.registries.borrow_mut();
                registries.hooks.entry(event).or_default().push(callback);
                Ok(())
            },
        );

        methods.add_method("execute_active_statement", |_lua, this, (): ()| {
            if let Some(tx) = &this.db_tx {
                let cancel = tokio_util::sync::CancellationToken::new();
                let _ = tx.send(DbCommand::ExecuteQuery {
                    sql: String::new(),
                    cancel,
                    auto_paginate: false,
                    page_size: 0,
                });
            }
            Ok(())
        });

        methods.add_method("print", |_lua, _this, msg: String| {
            info!("Lua print: {}", msg);
            Ok(())
        });
    }
}
