use std::time::Instant;

use ratatui::style::Color;

use crate::theme::IconMap;

#[derive(Debug, Clone)]
pub enum SchemaNode {
    Schema { name: String, expanded: bool, children: Vec<SchemaNode> },
    Table { schema: String, name: String, expanded: bool, loaded: bool, children: Vec<SchemaNode> },
    View { schema: String, name: String },
    Column { name: String, data_type: String, nullable: bool, is_primary_key: bool },
    Loading { schema: String, table: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Schema,
    Table,
    View,
    Column,
    Loading,
}

#[derive(Debug, Clone)]
pub struct FlatNode {
    pub depth: usize,
    pub kind: NodeKind,
    pub name: String,
    pub schema: Option<String>,
    pub table: Option<String>,
    pub data_type: Option<String>,
    pub nullable: bool,
    #[allow(dead_code)]
    pub is_primary_key: bool,
    pub expanded: bool,
    pub loaded: bool,
    pub icon: Option<(char, Color)>,
}

pub struct SchemaExplorer {
    pub tree: Vec<SchemaNode>,
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
            tree: Vec::new(),
            flat_view: Vec::new(),
            selected_idx: 0,
            icons: IconMap::darcula(),
            search_query: String::new(),
            search_active: false,
            last_key_time: Instant::now(),
            all_flat_nodes: Vec::new(),
        }
    }

    pub fn set_tree(&mut self, nodes: Vec<SchemaNode>) {
        self.tree = nodes;
        self.selected_idx = 0;
        self.search_query.clear();
        self.search_active = false;
        self.rebuild_flat_view();
    }

    pub fn rebuild_flat_view(&mut self) {
        let mut all_nodes = Vec::new();
        Self::flatten(&self.tree, 0, &mut all_nodes, &self.icons);
        self.all_flat_nodes = all_nodes;
        self.apply_search();
    }

    fn flatten(nodes: &[SchemaNode], depth: usize, output: &mut Vec<FlatNode>, icons: &IconMap) {
        for node in nodes {
            match node {
                SchemaNode::Schema { name, expanded, children } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Schema,
                        name: name.clone(),
                        schema: Some(name.clone()),
                        table: None,
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: *expanded,
                        loaded: true,
                        icon: Some(icons.schema),
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
                        schema: Some(schema.clone()),
                        table: Some(name.clone()),
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: *expanded,
                        loaded: *loaded,
                        icon: Some(icons.table),
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
                        schema: Some(schema.clone()),
                        table: Some(name.clone()),
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: false,
                        loaded: true,
                        icon: Some(icons.view),
                    });
                },
                SchemaNode::Column { name, data_type, nullable, is_primary_key } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Column,
                        name: name.clone(),
                        schema: None,
                        table: None,
                        data_type: Some(data_type.clone()),
                        nullable: *nullable,
                        is_primary_key: *is_primary_key,
                        expanded: false,
                        loaded: true,
                        icon: Some(icons.column),
                    });
                },
                SchemaNode::Loading { schema, table } => {
                    output.push(FlatNode {
                        depth,
                        kind: NodeKind::Loading,
                        name: "...".into(),
                        schema: Some(schema.clone()),
                        table: Some(table.clone()),
                        data_type: None,
                        nullable: false,
                        is_primary_key: false,
                        expanded: false,
                        loaded: false,
                        icon: None,
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
        let _node_info = &self.flat_view[flat_idx];
        let path = self.path_to_node(flat_idx);
        if let Some(target) = Self::get_node_mut(&mut self.tree, &path) {
            match target {
                SchemaNode::Schema { expanded, .. } => {
                    *expanded = true;
                },
                SchemaNode::Table { expanded, .. } => {
                    *expanded = true;
                },
                _ => {},
            }
        }
        self.rebuild_flat_view();
    }

    pub fn collapse_node(&mut self, flat_idx: usize) {
        if flat_idx >= self.flat_view.len() {
            return;
        }
        let path = self.path_to_node(flat_idx);
        if let Some(target) = Self::get_node_mut(&mut self.tree, &path) {
            match target {
                SchemaNode::Schema { expanded, children, .. } => {
                    *expanded = false;
                    Self::collapse_recursive(children);
                },
                SchemaNode::Table { expanded, .. } => {
                    *expanded = false;
                },
                _ => {},
            }
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
                _ => {},
            }
        }
    }

    fn path_to_node(&self, flat_idx: usize) -> Vec<usize> {
        let mut path = Vec::new();
        let mut count = 0;
        Self::build_path(&self.tree, flat_idx, &mut count, &mut path);
        path
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
                _ => {},
            }
        }
        false
    }

    pub fn insert_columns(&mut self, schema: &str, table: &str, columns: Vec<SchemaNode>) {
        Self::insert_columns_into(&mut self.tree, schema, table, columns);
        self.rebuild_flat_view();
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
                SchemaNode::Table { children, .. } => {
                    if Self::insert_columns_into(children, schema, table, columns.clone()) {
                        return true;
                    }
                },
                _ => {},
            }
        }
        false
    }

    pub fn set_loading_child(&mut self, schema: &str, table: &str) {
        Self::set_loading_in(&mut self.tree, schema, table);
        self.rebuild_flat_view();
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
                SchemaNode::Table { children, .. } => {
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
                _ => None,
            }
        }
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
