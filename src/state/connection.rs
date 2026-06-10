use crate::db::backend::EngineType;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting { dsn: String, masked: String },
    Connected { dsn: String, masked: String },
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectField {
    pub label: &'static str,
    pub value: String,
    pub cursor: usize,
    pub masked: bool,
    pub keychain_loaded: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectForm {
    pub fields: Vec<ConnectField>,
    pub active_field: usize,
    pub db_type: usize,
    pub selecting_type: bool,
    pub selected_profile: Option<usize>,
}

impl ConnectForm {
    pub fn default() -> Self {
        Self {
            fields: vec![
                ConnectField {
                    label: "Host",
                    value: "localhost".into(),
                    cursor: 9,
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Port",
                    value: "5432".into(),
                    cursor: 4,
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Database",
                    value: String::new(),
                    cursor: 0,
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "User",
                    value: String::new(),
                    cursor: 0,
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Password",
                    value: String::new(),
                    cursor: 0,
                    masked: true,
                    keychain_loaded: false,
                },
            ],
            active_field: 0,
            db_type: 0,
            selecting_type: true,
            selected_profile: None,
        }
    }

    pub fn from_profile(profile: &crate::config::ConnectionProfile) -> Self {
        let db_type = match profile.db_type.as_str() {
            "mysql" => 1,
            "sqlite" => 2,
            _ => 0,
        };
        let (password_value, keychain_loaded) = if profile.password_is_keychain() {
            match profile.get_password() {
                Ok(pass) => (pass, true),
                Err(_) => (String::new(), false),
            }
        } else {
            (String::new(), false)
        };
        Self {
            fields: vec![
                ConnectField {
                    label: "Host",
                    value: profile.host.clone(),
                    cursor: profile.host.len(),
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Port",
                    value: profile.port.to_string(),
                    cursor: profile.port.to_string().len(),
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Database",
                    value: profile.database.clone(),
                    cursor: profile.database.len(),
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "User",
                    value: profile.user.clone(),
                    cursor: profile.user.len(),
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Password",
                    value: password_value,
                    cursor: 0,
                    masked: true,
                    keychain_loaded,
                },
            ],
            active_field: 0,
            db_type,
            selecting_type: false,
            selected_profile: None,
        }
    }

    pub fn engine_type(&self) -> EngineType {
        match self.db_type {
            1 => EngineType::Mysql,
            2 => EngineType::Sqlite,
            _ => EngineType::Postgres,
        }
    }

    pub fn build_dsn(&self) -> String {
        match self.db_type {
            1 => self.build_mysql_dsn(),
            2 => self.build_sqlite_dsn(),
            _ => self.build_pg_dsn(),
        }
    }

    fn build_pg_dsn(&self) -> String {
        let host = &self.fields[0].value;
        let port = &self.fields[1].value;
        let db = &self.fields[2].value;
        let user = &self.fields[3].value;
        let pass = &self.fields[4].value;

        let mut dsn = String::from("postgresql://");

        let has_user = !user.is_empty();
        let has_pass = !pass.is_empty();

        if has_user || has_pass {
            dsn.push_str(user);
            if has_pass {
                dsn.push(':');
                dsn.push_str(pass);
            }
            dsn.push('@');
        }

        dsn.push_str(host);
        if !port.is_empty() {
            dsn.push(':');
            dsn.push_str(port);
        }
        if !db.is_empty() {
            dsn.push('/');
            dsn.push_str(db);
        }
        dsn
    }

    fn build_mysql_dsn(&self) -> String {
        let host = &self.fields[0].value;
        let port = &self.fields[1].value;
        let db = &self.fields[2].value;
        let user = &self.fields[3].value;
        let pass = &self.fields[4].value;

        let mut dsn = String::from("mysql://");

        let has_user = !user.is_empty();
        let has_pass = !pass.is_empty();

        if has_user || has_pass {
            dsn.push_str(if has_user { user } else { "root" });
            if has_pass {
                dsn.push(':');
                dsn.push_str(pass);
            }
            dsn.push('@');
        }

        dsn.push_str(host);
        if !port.is_empty() {
            dsn.push(':');
            dsn.push_str(port);
        }
        if !db.is_empty() {
            dsn.push('/');
            dsn.push_str(db);
        }
        dsn
    }

    fn build_sqlite_dsn(&self) -> String {
        let path = &self.fields[0].value;
        format!("sqlite://{path}")
    }

    pub fn masked_dsn(&self) -> String {
        match self.db_type {
            1 => self.masked_mysql_dsn(),
            2 => self.masked_sqlite_dsn(),
            _ => self.masked_pg_dsn(),
        }
    }

    fn masked_pg_dsn(&self) -> String {
        let host = &self.fields[0].value;
        let port = &self.fields[1].value;
        let db = &self.fields[2].value;
        let user = &self.fields[3].value;
        let pass = &self.fields[4].value;

        let mut dsn = String::from("postgresql://");

        let has_auth = !user.is_empty() || !pass.is_empty();
        if has_auth {
            dsn.push_str(user);
            if !pass.is_empty() {
                dsn.push_str(":***");
            }
            dsn.push('@');
        }

        dsn.push_str(host);
        if !port.is_empty() {
            dsn.push(':');
            dsn.push_str(port);
        }
        if !db.is_empty() {
            dsn.push('/');
            dsn.push_str(db);
        }
        dsn
    }

    fn masked_mysql_dsn(&self) -> String {
        let host = &self.fields[0].value;
        let port = &self.fields[1].value;
        let db = &self.fields[2].value;
        let user = &self.fields[3].value;
        let pass = &self.fields[4].value;

        let mut dsn = String::from("mysql://");

        let has_auth = !user.is_empty() || !pass.is_empty();
        if has_auth {
            dsn.push_str(if user.is_empty() { "root" } else { user });
            if !pass.is_empty() {
                dsn.push_str(":***");
            }
            dsn.push('@');
        }

        dsn.push_str(host);
        if !port.is_empty() {
            dsn.push(':');
            dsn.push_str(port);
        }
        if !db.is_empty() {
            dsn.push('/');
            dsn.push_str(db);
        }
        dsn
    }

    fn masked_sqlite_dsn(&self) -> String {
        let path = &self.fields[0].value;
        format!("sqlite://{path}")
    }
}
