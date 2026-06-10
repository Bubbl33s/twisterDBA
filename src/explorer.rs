use std::time::Instant;

use ratatui::style::Color;

use crate::db::backend::EngineType;
use crate::events::TableDetails;
use crate::state::ConnectionStatus;
use crate::theme::IconMap;

#[derive(Debug, Clone)]
pub enum SchemaNode {
    Schema { name: String, expanded: bool, children: Vec<SchemaNode> },
    Table { schema: String, name: String, expanded: bool, loaded: bool, children: Vec<SchemaNode> },
    View { schema: String, name: String },
    Column { name: String, data_type: String, nullable: bool, is_primary_key: bool },
    Loading { schema: String, table: String },
    Database { name: String, expanded: bool, children: Vec<SchemaNode> },
    ObjectFolder { kind: FolderKind, expanded: bool, loaded: bool, children: Vec<SchemaNode> },
    Index { name: String, columns: Vec<String>, is_unique: bool, is_primary: bool },
    ForeignKey { name: String, columns: Vec<String>, ref_table: String, ref_columns: Vec<String> },
    Key { name: String, columns: Vec<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FolderKind {
    Tables,
    Views,
    Columns,
    Keys,
    ForeignKeys,
    Indexes,
}

impl FolderKind {
    pub fn label(&self) -> &'static str {
        match self {
            FolderKind::Tables => "tables",
            FolderKind::Views => "views",
            FolderKind::Columns => "columns",
            FolderKind::Keys => "keys",
            FolderKind::ForeignKeys => "foreign_keys",
            FolderKind::Indexes => "indexes",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Source,
    Schema,
    Table,
    View,
    Column,
    Loading,
    Database,
    Folder,
    Index,
    ForeignKey,
    Key,
}

#[derive(Debug, Clone)]
pub struct DbSource {
    pub name: String,
    pub engine_type: EngineType,
    pub status: ConnectionStatus,
    #[allow(dead_code)]
    pub masked_dsn: String,
    pub tree: Vec<SchemaNode>,
    pub expanded: bool,
}

#[derive(Debug, Clone)]
pub struct FlatNode {
    pub depth: usize,
    pub kind: NodeKind,
    pub name: String,
    pub source_name: Option<String>,
    pub schema: Option<String>,
    pub table: Option<String>,
    pub data_type: Option<String>,
    pub nullable: bool,
    #[allow(dead_code)]
    pub is_primary_key: bool,
    pub expanded: bool,
    pub loaded: bool,
    pub expandable: bool,
    pub icon: Option<(char, Color)>,
    #[allow(dead_code)]
    pub folder_kind: Option<FolderKind>,
    pub columns: Option<Vec<String>>,
    pub ref_table: Option<String>,
    pub ref_columns: Option<Vec<String>>,
    pub is_unique: bool,
    pub child_count: usize,
}

enum NodePath {
    Source(usize),
    Tree(usize, Vec<usize>),
}

pub struct SchemaExplorer {
    pub sources: Vec<DbSource>,
    pub flat_view: Vec<FlatNode>,
    pub selected_idx: usize,
    icons: IconMap,
    pub search_query: String,
    pub search_active: bool,
    last_key_time: Instant,
    pub all_flat_nodes: Vec<FlatNode>,
}

impl SchemaExplorer {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            flat_view: Vec::new(),
            selected_idx: 0,
            icons: IconMap::darcula(),
            search_query: String::new(),
            search_active: false,
            last_key_time: Instant::now(),
            all_flat_nodes: Vec::new(),
        }
    }

    pub fn add_source(&mut self, source: DbSource) {
        if self.sources.iter().any(|s| s.name == source.name) {
            return;
        }
        self.sources.push(source);
        self.rebuild_flat_view();
    }

    pub fn remove_source(&mut self, name: &str) {
        self.sources.retain(|s| s.name != name);
        self.rebuild_flat_view();
    }

    pub fn set_source_status(&mut self, name: &str, status: ConnectionStatus) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == name) {
            source.status = status;
            self.rebuild_flat_view();
        }
    }

    pub fn expand_source(&mut self, name: &str) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == name) {
            source.expanded = true;
            self.rebuild_flat_view();
        }
    }

    pub fn collapse_source(&mut self, name: &str) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == name) {
            source.expanded = false;
            self.rebuild_flat_view();
        }
    }

    pub fn source(&self, name: &str) -> Option<&DbSource> {
        self.sources.iter().find(|s| s.name == name)
    }

    pub fn set_tree_for_source(&mut self, name: &str, nodes: Vec<SchemaNode>) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == name) {
            source.tree = nodes;
            self.search_query.clear();
            self.search_active = false;
            self.rebuild_flat_view();
        }
    }

    pub fn rebuild_flat_view(&mut self) {
        let mut all_nodes = Vec::new();
        for source in &self.sources {
            let engine_icon = self.engine_icon(source.engine_type);
            all_nodes.push(FlatNode {
                depth: 0,
                kind: NodeKind::Source,
                name: source.name.clone(),
                source_name: Some(source.name.clone()),
                schema: None,
                table: None,
                data_type: None,
                nullable: false,
                is_primary_key: false,
                expanded: source.expanded,
                loaded: true,
                expandable: true,
                icon: Some(engine_icon),
                folder_kind: None,
                columns: None,
                ref_table: None,
                ref_columns: None,
                is_unique: false,
                child_count: 0,
            });
            if source.expanded {
                Self::flatten(&source.tree, 1, &mut all_nodes, &self.icons);
            }
        }
        self.all_flat_nodes = all_nodes;
        self.apply_search();
    }

    fn engine_icon(&self, engine_type: EngineType) -> (char, Color) {
        match engine_type {
            EngineType::Postgres => self.icons.postgres,
            EngineType::Mysql => self.icons.mysql,
            EngineType::Sqlite => self.icons.sqlite,
        }
    }

    fn flatten(nodes: &[SchemaNode], depth: usize, output: &mut Vec<FlatNode>, icons: &IconMap) {
        for node in nodes {
            match node {
                SchemaNode::Schema { name, expanded, children } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Schema,
                        name: name.clone(),
                        source_name: None,
                        schema: Some(name.clone()),
                        table: None,
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: *expanded,
                        loaded: true,
                        expandable: true,
                        icon: Some(icons.schema),
                        folder_kind: None,
                        columns: None,
                        ref_table: None,
                        ref_columns: None,
                        is_unique: false,
                        child_count: children.len(),
                    });
                    if *expanded {
                        Self::flatten(children, depth + 1, output, icons);
                    }
                },
                SchemaNode::Table { schema, name, expanded, loaded, children } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Table,
                        name: name.clone(),
                        source_name: None,
                        schema: Some(schema.clone()),
                        table: Some(name.clone()),
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: *expanded,
                        loaded: *loaded,
                        expandable: true,
                        icon: Some(icons.table),
                        folder_kind: None,
                        columns: None,
                        ref_table: None,
                        ref_columns: None,
                        is_unique: false,
                        child_count: children.len(),
                    });
                    if *expanded {
                        Self::flatten(children, depth + 1, output, icons);
                    }
                },
                SchemaNode::View { schema, name } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::View,
                        name: name.clone(),
                        source_name: None,
                        schema: Some(schema.clone()),
                        table: Some(name.clone()),
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: false,
                        loaded: true,
                        expandable: false,
                        icon: Some(icons.view),
                        folder_kind: None,
                        columns: None,
                        ref_table: None,
                        ref_columns: None,
                        is_unique: false,
                        child_count: 0,
                    });
                },
                SchemaNode::Column { name, data_type, nullable, is_primary_key } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Column,
                        name: name.clone(),
                        source_name: None,
                        schema: None,
                        table: None,
                        data_type: Some(data_type.clone()),
                        nullable: *nullable,
                        is_primary_key: *is_primary_key,
                        expanded: false,
                        loaded: true,
                        expandable: false,
                        icon: Some(icons.column),
                        folder_kind: None,
                        columns: None,
                        ref_table: None,
                        ref_columns: None,
                        is_unique: false,
                        child_count: 0,
                    });
                },
                SchemaNode::Loading { schema, table } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Loading,
                        name: "...".into(),
                        source_name: None,
                        schema: Some(schema.clone()),
                        table: Some(table.clone()),
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: false,
                        loaded: false,
                        expandable: false,
                        icon: None,
                        folder_kind: None,
                        columns: None,
                        ref_table: None,
                        ref_columns: None,
                        is_unique: false,
                        child_count: 0,
                    });
                },
                SchemaNode::Database { name, expanded, children } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Database,
                        name: name.clone(),
                        source_name: None,
                        schema: None,
                        table: None,
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: *expanded,
                        loaded: true,
                        expandable: true,
                        icon: Some(icons.database),
                        folder_kind: None,
                        columns: None,
                        ref_table: None,
                        ref_columns: None,
                        is_unique: false,
                        child_count: children.len(),
                    });
                    if *expanded {
                        Self::flatten(children, depth + 1, output, icons);
                    }
                },
                SchemaNode::ObjectFolder { kind, expanded, loaded, children } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Folder,
                        name: kind.label().to_string(),
                        source_name: None,
                        schema: None,
                        table: None,
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: *expanded,
                        loaded: *loaded,
                        expandable: true,
                        icon: Some(icons.folder),
                        folder_kind: Some(kind.clone()),
                        columns: None,
                        ref_table: None,
                        ref_columns: None,
                        is_unique: false,
                        child_count: children.len(),
                    });
                    if *expanded {
                        Self::flatten(children, depth + 1, output, icons);
                    }
                },
                SchemaNode::Index { name, columns, is_unique, is_primary } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Index,
                        name: name.clone(),
                        source_name: None,
                        schema: None,
                        table: None,
                        data_type: None,
                        nullable: false,
                        is_primary_key: *is_primary,
                        expanded: false,
                        loaded: true,
                        expandable: false,
                        icon: Some(icons.index),
                        folder_kind: None,
                        columns: Some(columns.clone()),
                        ref_table: None,
                        ref_columns: None,
                        is_unique: *is_unique,
                        child_count: 0,
                    });
                },
                SchemaNode::ForeignKey { name, columns, ref_table, ref_columns } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::ForeignKey,
                        name: name.clone(),
                        source_name: None,
                        schema: None,
                        table: None,
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: false,
                        loaded: true,
                        expandable: false,
                        icon: Some(icons.foreign_key),
                        folder_kind: None,
                        columns: Some(columns.clone()),
                        ref_table: Some(ref_table.clone()),
                        ref_columns: Some(ref_columns.clone()),
                        is_unique: false,
                        child_count: 0,
                    });
                },
                SchemaNode::Key { name, columns } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Key,
                        name: name.clone(),
                        source_name: None,
                        schema: None,
                        table: None,
                        data_type: None,
                        nullable: false,
                        is_primary_key: true,
                        expanded: false,
                        loaded: true,
                        expandable: false,
                        icon: Some(icons.key),
                        folder_kind: None,
                        columns: Some(columns.clone()),
                        ref_table: None,
                        ref_columns: None,
                        is_unique: false,
                        child_count: 0,
                    });
                },
            }
        }
    }

    pub fn select_next(&mut self) {
        if self.flat_view.is_empty() {
            return;
        }
        self.selected_idx = (self.selected_idx + 1).min(self.flat_view.len() - 1);
    }

    pub fn select_prev(&mut self) {
        self.selected_idx = self.selected_idx.saturating_sub(1);
    }

    pub fn expand_node(&mut self, flat_idx: usize) {
        if flat_idx >= self.flat_view.len() {
            return;
        }
        let node_path = self.resolve_node_path(flat_idx);
        match node_path {
            Some(NodePath::Source(src_idx)) => {
                self.sources[src_idx].expanded = true;
            },
            Some(NodePath::Tree(src_idx, path)) => {
                if let Some(target) = Self::get_node_mut(&mut self.sources[src_idx].tree, &path) {
                    match target {
                        SchemaNode::Schema { expanded, .. } => *expanded = true,
                        SchemaNode::Table { expanded, .. } => *expanded = true,
                        SchemaNode::Database { expanded, .. } => *expanded = true,
                        SchemaNode::ObjectFolder { expanded, .. } => *expanded = true,
                        _ => {},
                    }
                }
            },
            None => {},
        }
        self.rebuild_flat_view();
    }

    pub fn collapse_node(&mut self, flat_idx: usize) {
        if flat_idx >= self.flat_view.len() {
            return;
        }
        let node_path = self.resolve_node_path(flat_idx);
        match node_path {
            Some(NodePath::Source(src_idx)) => {
                self.sources[src_idx].expanded = false;
            },
            Some(NodePath::Tree(src_idx, path)) => {
                if let Some(target) = Self::get_node_mut(&mut self.sources[src_idx].tree, &path) {
                    match target {
                        SchemaNode::Schema { expanded, children, .. } => {
                            *expanded = false;
                            Self::collapse_recursive(children);
                        },
                        SchemaNode::Table { expanded, .. } => {
                            *expanded = false;
                        },
                        SchemaNode::Database { expanded, children, .. } => {
                            *expanded = false;
                            Self::collapse_recursive(children);
                        },
                        SchemaNode::ObjectFolder { expanded, .. } => {
                            *expanded = false;
                        },
                        _ => {},
                    }
                }
            },
            None => {},
        }
        self.rebuild_flat_view();
    }

    fn collapse_recursive(nodes: &mut [SchemaNode]) {
        for node in nodes.iter_mut() {
            match node {
                SchemaNode::Schema { expanded, children, .. } => {
                    *expanded = false;
                    Self::collapse_recursive(children);
                },
                SchemaNode::Table { expanded, .. } => {
                    *expanded = false;
                },
                SchemaNode::Database { expanded, children, .. } => {
                    *expanded = false;
                    Self::collapse_recursive(children);
                },
                SchemaNode::ObjectFolder { expanded, .. } => {
                    *expanded = false;
                },
                _ => {},
            }
        }
    }

    fn resolve_node_path(&self, flat_idx: usize) -> Option<NodePath> {
        let mut count = 0;
        for (src_idx, source) in self.sources.iter().enumerate() {
            if count == flat_idx {
                return Some(NodePath::Source(src_idx));
            }
            count += 1;
            if source.expanded {
                let mut tree_path = Vec::new();
                if Self::build_path(&source.tree, flat_idx, &mut count, &mut tree_path) {
                    return Some(NodePath::Tree(src_idx, tree_path));
                }
            }
        }
        None
    }

    fn build_path(
        nodes: &[SchemaNode],
        target_idx: usize,
        count: &mut usize,
        path: &mut Vec<usize>,
    ) -> bool {
        for (i, node) in nodes.iter().enumerate() {
            if *count == target_idx {
                path.push(i);
                return true;
            }
            *count += 1;

            match node {
                SchemaNode::Schema { expanded, children, .. } => {
                    if *expanded {
                        path.push(i);
                        if Self::build_path(children, target_idx, count, path) {
                            return true;
                        }
                        path.pop();
                    }
                },
                SchemaNode::Table { expanded: true, children, .. } => {
                    path.push(i);
                    if Self::build_path(children, target_idx, count, path) {
                        return true;
                    }
                    path.pop();
                },
                SchemaNode::Table { .. } => {},
                SchemaNode::Database { expanded, children, .. } => {
                    if *expanded {
                        path.push(i);
                        if Self::build_path(children, target_idx, count, path) {
                            return true;
                        }
                        path.pop();
                    }
                },
                SchemaNode::ObjectFolder { expanded: true, children, .. } => {
                    path.push(i);
                    if Self::build_path(children, target_idx, count, path) {
                        return true;
                    }
                    path.pop();
                },
                SchemaNode::ObjectFolder { .. } => {},
                _ => {},
            }
        }
        false
    }

    pub fn insert_columns(
        &mut self,
        source_name: &str,
        schema: &str,
        table: &str,
        columns: Vec<SchemaNode>,
    ) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == source_name) {
            Self::insert_columns_into(&mut source.tree, schema, table, columns);
            self.rebuild_flat_view();
        }
    }

    fn insert_columns_into(
        nodes: &mut [SchemaNode],
        schema: &str,
        table: &str,
        columns: Vec<SchemaNode>,
    ) -> bool {
        for node in nodes.iter_mut() {
            match node {
                SchemaNode::Table { schema: s, name: t, loaded, children, .. }
                    if s == schema && t == table =>
                {
                    *loaded = true;
                    *children = columns;
                    return true;
                },
                SchemaNode::Schema { children, .. } => {
                    if Self::insert_columns_into(children, schema, table, columns.clone()) {
                        return true;
                    }
                },
                SchemaNode::Database { children, .. } => {
                    if Self::insert_columns_into(children, schema, table, columns.clone()) {
                        return true;
                    }
                },
                SchemaNode::Table { children, .. } => {
                    if Self::insert_columns_into(children, schema, table, columns.clone()) {
                        return true;
                    }
                },
                SchemaNode::ObjectFolder { children, .. } => {
                    if Self::insert_columns_into(children, schema, table, columns.clone()) {
                        return true;
                    }
                },
                _ => {},
            }
        }
        false
    }

    pub fn set_loading_child(&mut self, source_name: &str, schema: &str, table: &str) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == source_name) {
            Self::set_loading_in(&mut source.tree, schema, table);
            self.rebuild_flat_view();
        }
    }

    fn set_loading_in(nodes: &mut [SchemaNode], schema: &str, table: &str) -> bool {
        for node in nodes.iter_mut() {
            match node {
                SchemaNode::Table { schema: s, name: t, children, .. }
                    if s == schema && t == table =>
                {
                    *children = vec![SchemaNode::Loading {
                        schema: schema.to_string(),
                        table: table.to_string(),
                    }];
                    return true;
                },
                SchemaNode::Schema { children, .. } => {
                    if Self::set_loading_in(children, schema, table) {
                        return true;
                    }
                },
                SchemaNode::Database { children, .. } => {
                    if Self::set_loading_in(children, schema, table) {
                        return true;
                    }
                },
                SchemaNode::Table { children, .. } => {
                    if Self::set_loading_in(children, schema, table) {
                        return true;
                    }
                },
                SchemaNode::ObjectFolder { children, .. } => {
                    if Self::set_loading_in(children, schema, table) {
                        return true;
                    }
                },
                _ => {},
            }
        }
        false
    }

    pub fn node_at(&self, flat_idx: usize) -> Option<&FlatNode> {
        self.flat_view.get(flat_idx)
    }

    pub fn node_kind_at(&self, flat_idx: usize) -> Option<NodeKind> {
        self.node_at(flat_idx).map(|n| n.kind.clone())
    }

    pub fn node_loaded_at(&self, flat_idx: usize) -> bool {
        self.node_at(flat_idx).is_some_and(|n| n.loaded)
    }

    pub fn node_expanded_at(&self, flat_idx: usize) -> bool {
        self.node_at(flat_idx).is_some_and(|n| n.expanded)
    }

    pub fn node_schema_at(&self, flat_idx: usize) -> Option<String> {
        self.node_at(flat_idx).and_then(|n| n.schema.clone())
    }

    pub fn node_table_at(&self, flat_idx: usize) -> Option<String> {
        self.node_at(flat_idx).and_then(|n| n.table.clone())
    }

    pub fn node_source_name_at(&self, flat_idx: usize) -> Option<String> {
        self.node_at(flat_idx).and_then(|n| n.source_name.clone())
    }

    fn get_node_mut<'a>(nodes: &'a mut [SchemaNode], path: &[usize]) -> Option<&'a mut SchemaNode> {
        if path.is_empty() {
            return None;
        }
        let idx = path[0];
        let node = nodes.get_mut(idx)?;
        if path.len() == 1 {
            Some(node)
        } else {
            match node {
                SchemaNode::Schema { children, .. } => Self::get_node_mut(children, &path[1..]),
                SchemaNode::Table { children, .. } => Self::get_node_mut(children, &path[1..]),
                SchemaNode::Database { children, .. } => Self::get_node_mut(children, &path[1..]),
                SchemaNode::ObjectFolder { children, .. } => {
                    Self::get_node_mut(children, &path[1..])
                },
                _ => None,
            }
        }
    }

    pub fn insert_table_details(
        &mut self,
        source_name: &str,
        schema: &str,
        table: &str,
        details: TableDetails,
    ) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == source_name) {
            Self::insert_table_details_into(&mut source.tree, schema, table, details);
            self.rebuild_flat_view();
        }
    }

    fn insert_table_details_into(
        nodes: &mut [SchemaNode],
        schema: &str,
        table: &str,
        details: TableDetails,
    ) -> bool {
        for node in nodes.iter_mut() {
            match node {
                SchemaNode::Table { schema: s, name: t, loaded, children, .. }
                    if s == schema && t == table =>
                {
                    *loaded = true;
                    let mut folders = Vec::new();

                    let column_nodes: Vec<SchemaNode> = details
                        .columns
                        .iter()
                        .map(|c| SchemaNode::Column {
                            name: c.name.clone(),
                            data_type: c.data_type.clone(),
                            nullable: c.nullable,
                            is_primary_key: c.is_primary_key,
                        })
                        .collect();
                    if !column_nodes.is_empty() {
                        folders.push(SchemaNode::ObjectFolder {
                            kind: FolderKind::Columns,
                            expanded: false,
                            loaded: true,
                            children: column_nodes,
                        });
                    }

                    let key_nodes: Vec<SchemaNode> = details
                        .keys
                        .iter()
                        .map(|k| SchemaNode::Key {
                            name: k.name.clone(),
                            columns: k.columns.clone(),
                        })
                        .collect();
                    if !key_nodes.is_empty() {
                        folders.push(SchemaNode::ObjectFolder {
                            kind: FolderKind::Keys,
                            expanded: false,
                            loaded: true,
                            children: key_nodes,
                        });
                    }

                    let fk_nodes: Vec<SchemaNode> = details
                        .foreign_keys
                        .iter()
                        .map(|fk| SchemaNode::ForeignKey {
                            name: fk.name.clone(),
                            columns: fk.columns.clone(),
                            ref_table: fk.ref_table.clone(),
                            ref_columns: fk.ref_columns.clone(),
                        })
                        .collect();
                    if !fk_nodes.is_empty() {
                        folders.push(SchemaNode::ObjectFolder {
                            kind: FolderKind::ForeignKeys,
                            expanded: false,
                            loaded: true,
                            children: fk_nodes,
                        });
                    }

                    let index_nodes: Vec<SchemaNode> = details
                        .indexes
                        .iter()
                        .map(|idx| SchemaNode::Index {
                            name: idx.name.clone(),
                            columns: idx.columns.clone(),
                            is_unique: idx.is_unique,
                            is_primary: idx.is_primary,
                        })
                        .collect();
                    if !index_nodes.is_empty() {
                        folders.push(SchemaNode::ObjectFolder {
                            kind: FolderKind::Indexes,
                            expanded: false,
                            loaded: true,
                            children: index_nodes,
                        });
                    }

                    *children = folders;
                    return true;
                },
                SchemaNode::Schema { children, .. } => {
                    if Self::insert_table_details_into(children, schema, table, details.clone()) {
                        return true;
                    }
                },
                SchemaNode::Database { children, .. } => {
                    if Self::insert_table_details_into(children, schema, table, details.clone()) {
                        return true;
                    }
                },
                SchemaNode::ObjectFolder { children, .. } => {
                    if Self::insert_table_details_into(children, schema, table, details.clone()) {
                        return true;
                    }
                },
                SchemaNode::Table { children, .. } => {
                    if Self::insert_table_details_into(children, schema, table, details.clone()) {
                        return true;
                    }
                },
                _ => {},
            }
        }
        false
    }

    pub fn apply_search(&mut self) {
        if self.search_query.is_empty() {
            self.flat_view = self.all_flat_nodes.clone();
            self.search_active = false;
        } else {
            let query = self.search_query.to_lowercase();
            self.flat_view = self
                .all_flat_nodes
                .iter()
                .filter(|node| node.name.to_lowercase().contains(&query))
                .cloned()
                .collect();
            self.search_active = true;
        }
        if self.selected_idx >= self.flat_view.len() && !self.flat_view.is_empty() {
            self.selected_idx = self.flat_view.len() - 1;
        }
        if self.flat_view.is_empty() {
            self.selected_idx = 0;
        }
    }

    pub fn push_search_char(&mut self, c: char) {
        let now = Instant::now();
        if self.last_key_time.elapsed() > std::time::Duration::from_secs(1) {
            self.search_query.clear();
        }
        self.last_key_time = now;
        self.search_query.push(c);
        self.apply_search();
    }

    pub fn pop_search_char(&mut self) {
        self.search_query.pop();
        self.apply_search();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.apply_search();
    }
}
