use std::fs::File;
use std::io::{self, BufRead};

#[derive(Debug, PartialEq, Clone)]
pub enum PortMode {
    In,
    Out,
    InOut,
    Buffer,
    Linkage,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Port {
    pub name: String,
    pub mode: PortMode,
    pub data_type: String,
    pub line: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Entity {
    pub name: String,
    pub ports: Vec<Port>,
    pub line: usize,
    pub instantiations: Vec<String>,
}

pub struct VhdlProject {
    pub entities: Vec<Entity>,
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
        self.parse_reader(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn parse_reader<R: BufRead>(&mut self, reader: R) -> Result<(), String> {
        let mut current_entity: Option<Entity> = None;
        let mut current_arch_entity_idx: Option<usize> = None;
        let mut in_port_block = false;

        for (line_idx, line_result) in reader.lines().enumerate() {
            let line_num = line_idx + 1;
            let raw_line = match line_result {
                Ok(s) => s,
                Err(e) => return Err(format!("Error reading line: {}", e)),
            };

            let line_str = raw_line.trim().to_lowercase();

            // Skip comments and empty lines
            if line_str.is_empty() || line_str.starts_with("--") {
                continue;
            }

            // Remove inline comments for parsing
            let line_str_no_comment = if let Some(idx) = line_str.find("--") {
                &line_str[..idx]
            } else {
                &line_str
            };
            let line_str_no_comment = line_str_no_comment.trim();

            let raw_line_no_comment = if let Some(idx) = raw_line.find("--") {
                &raw_line[..idx]
            } else {
                &raw_line
            };
            let raw_line_no_comment = raw_line_no_comment.trim();
            let raw_line_lower = raw_line_no_comment.to_lowercase();

            // Detect entity start
            if line_str_no_comment.starts_with("entity ") && line_str_no_comment.contains(" is") {
                let parts: Vec<&str> = raw_line_no_comment.split_whitespace().collect();
                if parts.len() >= 3 && parts[2].to_lowercase() == "is" {
                    current_entity = Some(Entity {
                        name: parts[1].to_string(),
                        ports: Vec::new(),
                        line: line_num,
                        instantiations: Vec::new(),
                    });
                }
            } else if let Some(ref mut entity) = current_entity {
                // Detect entity end
                if line_str_no_comment.starts_with("end entity")
                    || line_str_no_comment
                        .starts_with(&format!("end {}", entity.name.to_lowercase()))
                    || line_str_no_comment.starts_with("end;")
                    || line_str_no_comment == format!("end {};", entity.name.to_lowercase())
                    || line_str_no_comment == format!("end {}", entity.name.to_lowercase())
                {
                    self.entities.push(current_entity.take().unwrap());
                    in_port_block = false;
                } else {
                    if line_str_no_comment.contains("port (")
                        || line_str_no_comment.contains("port(")
                    {
                        in_port_block = true;
                    }

                    if in_port_block {
                        let port_part = if let Some(idx) = raw_line_lower.find("port (") {
                            &raw_line_no_comment[idx + 6..]
                        } else if let Some(idx) = raw_line_lower.find("port(") {
                            &raw_line_no_comment[idx + 5..]
                        } else {
                            raw_line_no_comment
                        };

                        let parts: Vec<&str> = port_part.split(':').collect();
                        if parts.len() == 2 {
                            let name_part = parts[0].trim();
                            let type_part = parts[1]
                                .trim()
                                .trim_end_matches(';')
                                .trim_end_matches(')')
                                .trim();

                            let type_tokens: Vec<&str> = type_part.split_whitespace().collect();
                            if !type_tokens.is_empty() {
                                let mode_str = type_tokens[0].to_lowercase();
                                let mode = match mode_str.as_str() {
                                    "in" => Some(PortMode::In),
                                    "out" => Some(PortMode::Out),
                                    "inout" => Some(PortMode::InOut),
                                    "buffer" => Some(PortMode::Buffer),
                                    "linkage" => Some(PortMode::Linkage),
                                    _ => None,
                                };

                                if let Some(valid_mode) = mode {
                                    let data_type = type_tokens[1..].join(" ");

                                    let names: Vec<&str> = name_part.split(',').collect();
                                    for name in names {
                                        let clean_name = name.trim();
                                        if !clean_name.is_empty() {
                                            entity.ports.push(Port {
                                                name: clean_name.to_string(),
                                                mode: valid_mode.clone(),
                                                data_type: data_type.clone(),
                                                line: line_num,
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        if line_str_no_comment.ends_with(");") || line_str_no_comment == ");" {
                            in_port_block = false;
                        }
                    }
                }
            } else {
                // Not inside an entity definition, could be an architecture or instantiation
                if line_str_no_comment.starts_with("architecture ")
                    && line_str_no_comment.contains(" of ")
                {
                    let parts: Vec<&str> = raw_line_no_comment.split_whitespace().collect();
                    if parts.len() >= 4 && parts[2].to_lowercase() == "of" {
                        let target_entity = parts[3].trim_end_matches("is").trim();
                        // Find the entity in self.entities
                        current_arch_entity_idx = self
                            .entities
                            .iter()
                            .position(|e| e.name.eq_ignore_ascii_case(target_entity));
                    }
                } else if line_str_no_comment.starts_with("end architecture")
                    || line_str_no_comment.starts_with("end behavioral")
                    || line_str_no_comment.starts_with("end rtl")
                {
                    // Very naive architecture end tracking, but it works for matching bounded scope
                    current_arch_entity_idx = None;
                } else if let Some(arch_idx) = current_arch_entity_idx {
                    // Inside architecture body, look for instantiations
                    if line_str_no_comment.contains(":") {
                        let parts: Vec<&str> = raw_line_no_comment.split(':').collect();
                        if parts.len() >= 2 {
                            let right_side = parts[1..].join(":").trim().to_string();
                            let right_side_lower = right_side.to_lowercase();

                            let mut instantiated_entity = None;

                            if right_side_lower.starts_with("entity work.") {
                                let after_work = &right_side[12..]; // skip "entity work."
                                let tokens: Vec<&str> = after_work.split_whitespace().collect();
                                if !tokens.is_empty() {
                                    // Could be "entity work.MyEntity generic map ..." or just "entity work.MyEntity"
                                    instantiated_entity = Some(tokens[0].to_string());
                                }
                            } else if right_side_lower.contains("port map")
                                || right_side_lower.contains("generic map")
                            {
                                // Likely standard component instantiation, e.g., "my_inst: MyComponent port map (...)"
                                let tokens: Vec<&str> = right_side.split_whitespace().collect();
                                if !tokens.is_empty()
                                    && !tokens[0].eq_ignore_ascii_case("port")
                                    && !tokens[0].eq_ignore_ascii_case("generic")
                                {
                                    instantiated_entity = Some(tokens[0].to_string());
                                }
                            }

                            if let Some(mut name) = instantiated_entity {
                                // Clean up the name in case it ends with port map keywords or parenthesis if it was unspaced
                                if let Some(idx) = name.find('(') {
                                    name = name[..idx].to_string();
                                }
                                if !self.entities[arch_idx].instantiations.contains(&name) {
                                    self.entities[arch_idx].instantiations.push(name);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn print_hierarchy(&self) {
        // Find top level entities (entities not instantiated by anyone else)
        let mut all_instantiated = Vec::new();
        for entity in &self.entities {
            for inst in &entity.instantiations {
                if !all_instantiated.contains(&inst.to_lowercase()) {
                    all_instantiated.push(inst.to_lowercase());
                }
            }
        }

        let mut roots = Vec::new();
        for entity in &self.entities {
            if !all_instantiated.contains(&entity.name.to_lowercase()) {
                roots.push(entity);
            }
        }

        if roots.is_empty() {
            println!("No top level entity found (possible circular dependency or no entities).");
        } else {
            for root in roots {
                let mut path = Vec::new();
                self.print_entity_tree(root, 0, &mut path);
            }
        }
    }

    fn print_entity_tree<'a>(&'a self, entity: &'a Entity, depth: usize, path: &mut Vec<&'a str>) {
        let indent = " ".repeat(depth * 4);

        if path.contains(&entity.name.as_str()) {
            println!(
                "{}Entity: {} (Line {}) [Circular Dependency Detected]",
                indent, entity.name, entity.line
            );
            return;
        }

        path.push(&entity.name);

        println!("{}Entity: {} (Line {})", indent, entity.name, entity.line);

        let port_indent = " ".repeat(depth * 4 + 2);
        for port in &entity.ports {
            println!(
                "{}Port: {} : {:?} {} (Line {})",
                port_indent, port.name, port.mode, port.data_type, port.line
            );
        }

        for inst_name in &entity.instantiations {
            if let Some(child_entity) = self
                .entities
                .iter()
                .find(|e| e.name.eq_ignore_ascii_case(inst_name))
            {
                self.print_entity_tree(child_entity, depth + 1, path);
            } else {
                let child_indent = " ".repeat((depth + 1) * 4);
                println!("{}[External/Unknown Entity: {}]", child_indent, inst_name);
            }
        }

        path.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vhdl_project_hierarchy() {
        let vhdl_content = "
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity AND_Gate is
    Port ( A : in  STD_LOGIC;
           B : in  STD_LOGIC;
           Y : out STD_LOGIC);
end AND_Gate;

entity OR_Gate is
    Port ( A : in  STD_LOGIC;
           B : in  STD_LOGIC;
           Y : out STD_LOGIC);
end OR_Gate;

entity Top_Level is
    Port ( In1 : in STD_LOGIC;
           In2 : in STD_LOGIC;
           Out1 : out STD_LOGIC);
end Top_Level;

architecture Structural of Top_Level is
begin
    inst_and: entity work.AND_Gate
        port map (A => In1, B => In2, Y => Out1);
        
    inst_or: OR_Gate generic map (delay => 5ns) port map (A => In1, B => In2, Y => Out1);
end Structural;
";
        let mut project = VhdlProject::new();
        project.parse_reader(vhdl_content.as_bytes()).unwrap();

        assert_eq!(project.entities.len(), 3);

        let and_gate = project
            .entities
            .iter()
            .find(|e| e.name == "AND_Gate")
            .unwrap();
        assert_eq!(and_gate.ports.len(), 3);

        let top_level = project
            .entities
            .iter()
            .find(|e| e.name == "Top_Level")
            .unwrap();
        assert_eq!(top_level.instantiations.len(), 2);
        assert!(top_level.instantiations.contains(&"AND_Gate".to_string()));
        assert!(top_level.instantiations.contains(&"OR_Gate".to_string()));
    }
}
