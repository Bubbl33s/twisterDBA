use crate::db::backend::EngineType;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting { dsn: String, masked: String },
    Connected { dsn: String, masked: String },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ConnectionEntry {
    pub name: String,
    #[allow(dead_code)]
    pub engine_type: EngineType,
    pub status: ConnectionStatus,
    pub masked_dsn: String,
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
pub enum DialogStep {
    SelectType,
    EnterDetails,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectForm {
    pub step: DialogStep,
    pub fields: Vec<ConnectField>,
    pub active_field: usize,
    pub db_type: usize,
    pub selected_profile: Option<usize>,
    pub connection_name: String,
    pub connection_name_cursor: usize,
    pub ssl_mode: usize,
    pub type_cursor: usize,
    pub name_conflict: bool,
}

impl ConnectForm {
    pub const SSL_MODES: &[&str] =
        &["disable", "allow", "prefer", "require", "verify-ca", "verify-full"];

    pub fn default() -> Self {
        Self {
            step: DialogStep::SelectType,
            fields: Self::fields_for_engine(0),
            active_field: 0,
            db_type: 0,
            selected_profile: None,
            connection_name: String::new(),
            connection_name_cursor: 0,
            ssl_mode: 2,
            type_cursor: 0,
            name_conflict: false,
        }
    }

    pub fn fields_for_engine_pub(db_type: usize) -> Vec<ConnectField> {
        Self::fields_for_engine(db_type)
    }

    fn fields_for_engine(db_type: usize) -> Vec<ConnectField> {
        match db_type {
            1 => vec![
                ConnectField {
                    label: "Host",
                    value: "localhost".into(),
                    cursor: 9,
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Port",
                    value: "3306".into(),
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
            2 => vec![ConnectField {
                label: "File Path",
                value: String::new(),
                cursor: 0,
                masked: false,
                keychain_loaded: false,
            }],
            _ => vec![
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
        let fields = match db_type {
            2 => vec![ConnectField {
                label: "File Path",
                value: profile.host.clone(),
                cursor: profile.host.len(),
                masked: false,
                keychain_loaded: false,
            }],
            _ => vec![
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
        };
        let connection_name = profile.name.clone();
        Self {
            step: DialogStep::EnterDetails,
            fields,
            active_field: 0,
            db_type,
            selected_profile: None,
            connection_name_cursor: connection_name.len(),
            connection_name,
            ssl_mode: 2,
            type_cursor: db_type,
            name_conflict: false,
        }
    }

    #[allow(dead_code)]
    pub fn from_profile_with_name(
        profile: &crate::config::ConnectionProfile,
        name: String,
    ) -> Self {
        let mut form = Self::from_profile(profile);
        form.connection_name = name.clone();
        form.connection_name_cursor = name.len();
        form
    }

    pub fn engine_type(&self) -> EngineType {
        match self.db_type {
            1 => EngineType::Mysql,
            2 => EngineType::Sqlite,
            _ => EngineType::Postgres,
        }
    }

    pub fn auto_generate_name(&self) -> String {
        let engine_prefix = match self.db_type {
            1 => "mysql",
            2 => "sqlite",
            _ => "postgres",
        };
        let host = self.fields.first().map(|f| f.value.as_str()).unwrap_or("localhost");
        if host.is_empty() {
            format!("{}-localhost", engine_prefix)
        } else {
            format!("{}-{}", engine_prefix, host)
        }
    }

    pub fn total_field_count(&self) -> usize {
        let base = 1 + self.fields.len();
        if self.db_type == 0 { base + 1 } else { base }
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
        let ssl_mode = Self::SSL_MODES[self.ssl_mode];
        dsn.push_str(&format!("?sslmode={}", ssl_mode));
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
        let ssl_mode = Self::SSL_MODES[self.ssl_mode];
        dsn.push_str(&format!("?sslmode={}", ssl_mode));
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
