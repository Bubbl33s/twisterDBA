use std::collections::HashMap;

#[derive(Default)]
pub struct LuaRegistries {
    pub keymaps: HashMap<String, mlua::Function>,
    pub commands: HashMap<String, mlua::Function>,
    pub hooks: HashMap<String, Vec<mlua::Function>>,
    pub extractors: HashMap<String, mlua::Function>,
    pub theme_overrides: HashMap<String, String>,
}
