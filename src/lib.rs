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
}

pub struct VhdlFile {
    pub path: String,
    pub entities: Vec<Entity>,
}

impl VhdlFile {
    pub fn open(path: &str) -> Result<Self, io::Error> {
        let mut vhdl_file = Self {
            path: path.to_string(),
            entities: Vec::new(),
        };
        vhdl_file.parse_file()?;
        Ok(vhdl_file)
    }

    fn parse_file(&mut self) -> Result<(), io::Error> {
        let file = File::open(&self.path)?;
        let reader = io::BufReader::new(file);
        self.parse_reader(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn parse_reader<R: BufRead>(&mut self, reader: R) -> Result<(), String> {
        let mut current_entity: Option<Entity> = None;
        let mut in_port_block = false;

        for (line_idx, line_result) in reader.lines().enumerate() {
            let line_num = line_idx + 1;
            let raw_line = match line_result {
                Ok(s) => s,
                Err(_) => continue,
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

            if line_str_no_comment.starts_with("entity ") && line_str_no_comment.contains(" is") {
                let parts: Vec<&str> = raw_line_no_comment.split_whitespace().collect();
                if parts.len() >= 3 && parts[2].to_lowercase() == "is" {
                    current_entity = Some(Entity {
                        name: parts[1].to_string(),
                        ports: Vec::new(),
                        line: line_num,
                    });
                }
            } else if let Some(ref mut entity) = current_entity {
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
                        let port_part = if let Some(idx) =
                            raw_line_no_comment.to_lowercase().find("port (")
                        {
                            &raw_line_no_comment[idx + 6..]
                        } else if let Some(idx) = raw_line_no_comment.to_lowercase().find("port(") {
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
            }
        }

        Ok(())
    }

    pub fn print_entities(&self) {
        println!("Parsed VHDL File: {}", self.path);
        for entity in &self.entities {
            println!("Entity: {} (Line {})", entity.name, entity.line);
            for port in &entity.ports {
                println!(
                    "  Port: {} : {:?} {} (Line {})",
                    port.name, port.mode, port.data_type, port.line
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vhdl_file_parse_reader() {
        let vhdl_content = "
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity AND_Gate is
    Port ( A : in  STD_LOGIC;
           B : in  STD_LOGIC;
           Y : out STD_LOGIC);
end AND_Gate;
";
        let mut vhdl_file = VhdlFile {
            path: "test.vhd".to_string(),
            entities: vec![],
        };
        vhdl_file.parse_reader(vhdl_content.as_bytes()).unwrap();

        assert_eq!(vhdl_file.entities.len(), 1);

        let entity = &vhdl_file.entities[0];
        assert_eq!(entity.name, "AND_Gate");
        assert_eq!(entity.line, 5); // Empty string is line 1, first newline starts line 2 etc.
        assert_eq!(entity.ports.len(), 3);

        assert_eq!(entity.ports[0].name, "A");
        assert_eq!(entity.ports[0].mode, PortMode::In);
        assert_eq!(entity.ports[0].data_type, "STD_LOGIC");
        assert_eq!(entity.ports[0].line, 6);

        assert_eq!(entity.ports[1].name, "B");
        assert_eq!(entity.ports[1].mode, PortMode::In);
        assert_eq!(entity.ports[1].data_type, "STD_LOGIC");
        assert_eq!(entity.ports[1].line, 7);

        assert_eq!(entity.ports[2].name, "Y");
        assert_eq!(entity.ports[2].mode, PortMode::Out);
        assert_eq!(entity.ports[2].data_type, "STD_LOGIC");
        assert_eq!(entity.ports[2].line, 8);
    }
}
