use serde::Serialize;
use std::fs::File;
use std::io::{self, BufRead};

use crate::vhdl_entity::VhdlEntity;
use crate::vhdl_token::VhdlToken;

#[derive(Debug, Serialize)]
pub struct VhdlProject {
    pub entities: Vec<VhdlEntity>,
}

impl Default for VhdlProject {
    fn default() -> Self {
        Self::new()
    }
}

impl VhdlProject {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn parse_file(&mut self, path: &str) -> Result<(), io::Error> {
        let metadata = std::fs::metadata(path)?;
        if metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Is a directory",
            ));
        }
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        self.parse_reader(reader, path)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn parse_reader<R: BufRead>(
        &mut self,
        mut reader: R,
        file_name: &str,
    ) -> Result<(), String> {
        let mut content = String::new();
        if let Err(e) = reader.read_to_string(&mut content) {
            return Err(format!("Error reading file: {}", e));
        }

        let tokens = VhdlToken::tokenize(&content);
        VhdlEntity::parse_entities(&mut self.entities, &tokens, file_name);
        Ok(())
    }

    /// Build a hierarchical JSON tree from the flat parsed entities.
    /// Returns only top-level (root) entities, with children nested.
    pub fn export_json_tree(&self) -> Vec<VhdlEntity> {
        let mut all_instantiated = Vec::new();
        for entity in &self.entities {
            for arch in &entity.architectures {
                for child in &arch.children {
                    let name_lc = child.name.to_lowercase();
                    if !all_instantiated.contains(&name_lc) {
                        all_instantiated.push(name_lc);
                    }
                }
            }
        }

        let mut roots = Vec::new();
        for entity in &self.entities {
            if !all_instantiated.contains(&entity.name.to_lowercase()) {
                roots.push(self.resolve_entity(entity, &mut Vec::new()));
            }
        }
        roots
    }

    /// Recursively resolve an entity: clone it with its children resolved
    fn resolve_entity<'a>(&'a self, entity: &'a VhdlEntity, path: &mut Vec<&'a str>) -> VhdlEntity {
        if path.contains(&entity.name.as_str()) {
            return entity.clone();
        }

        path.push(&entity.name);

        let mut resolved = entity.clone();
        for arch in &mut resolved.architectures {
            let mut resolved_children = Vec::new();
            for child_stub in &arch.children {
                if let Some(full_entity) = self
                    .entities
                    .iter()
                    .find(|e| e.name.eq_ignore_ascii_case(&child_stub.name))
                {
                    let mut resolved_child = self.resolve_entity(full_entity, path);
                    for stub_gen in &child_stub.generics {
                        if let Some(resolved_gen) = resolved_child
                            .generics
                            .iter_mut()
                            .find(|g| g.name.eq_ignore_ascii_case(&stub_gen.name))
                        {
                            resolved_gen.actual_value = stub_gen.actual_value.clone();
                        }
                    }
                    resolved_children.push(resolved_child);
                } else {
                    resolved_children.push(child_stub.clone());
                }
            }
            arch.children = resolved_children;
        }

        path.pop();
        resolved
    }
}
