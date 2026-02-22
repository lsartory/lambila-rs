use std::fs::File;
use std::io::{self, BufRead};

fn extract_last_number(s: &str) -> Option<i32> {
    let mut num_str = String::new();
    for c in s.chars().rev() {
        if c.is_ascii_digit() {
            num_str.push(c);
        } else if !num_str.is_empty() {
            if c == '-' {
                num_str.push('-');
            }
            break;
        }
    }
    if num_str.is_empty() || num_str == "-" {
        None
    } else {
        num_str.chars().rev().collect::<String>().parse().ok()
    }
}

fn extract_first_number(s: &str) -> Option<i32> {
    let mut num_str = String::new();
    let mut in_num = false;
    for c in s.chars() {
        if c.is_ascii_digit() || (c == '-' && !in_num) {
            num_str.push(c);
            in_num = true;
        } else if in_num {
            break;
        }
    }
    if num_str.is_empty() || num_str == "-" {
        None
    } else {
        num_str.parse().ok()
    }
}

fn parse_range_size(range_expr: &str, entity: Option<&Entity>) -> usize {
    let expr = range_expr.to_lowercase();

    // Check for 'range
    if let Some(idx) = expr.find("'range") {
        let before_range = &expr[..idx];
        let tokens: Vec<&str> = before_range.split_whitespace().collect();
        if let Some(&signal_name) = tokens.last() {
            if let Some(ent) = entity {
                // Find the signal in ports
                if let Some(port) = ent
                    .ports
                    .iter()
                    .find(|p| p.name.to_lowercase() == signal_name)
                {
                    // Use the dedicated 'range' parameter if it's available, parsing limits exclusively!
                    if let Some(ref r) = port.range {
                        let dir_str = match r.direction {
                            Direction::To => "to",
                            Direction::Downto => "downto",
                        };
                        return parse_range_size(
                            &format!("{} {} {}", r.left, dir_str, r.right),
                            None,
                        );
                    } else {
                        return parse_range_size(&port.data_type, None);
                    }
                }
            }
        }
        return 1; // Fallback
    }

    let parts: Vec<&str> = if expr.contains(" downto ") {
        expr.split(" downto ").collect()
    } else if expr.contains(" to ") {
        expr.split(" to ").collect()
    } else {
        Vec::new()
    };

    if parts.len() >= 2 {
        let left = extract_last_number(parts[0]);
        let right = extract_first_number(parts[1]);
        if let (Some(l), Some(r)) = (left, right) {
            return (l - r).abs() as usize + 1;
        }
    }

    1
}

#[derive(Debug, PartialEq, Clone)]
pub enum PortMode {
    In,
    Out,
    InOut,
    Buffer,
    Linkage,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Direction {
    To,
    Downto,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Range {
    pub left: String,
    pub right: String,
    pub direction: Direction,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Port {
    pub name: String,
    pub mode: PortMode,
    pub data_type: String,    // E.g., std_logic, std_logic_vector
    pub range: Option<Range>, // E.g., 7 downto 0
    pub file_name: String,
    pub line: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Instantiation {
    pub entity_name: String,
    pub ports: Vec<Port>,
    pub file_name: String,
    pub line: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Entity {
    pub name: String,
    pub ports: Vec<Port>,
    pub file_name: String,
    pub line: usize,
    pub instantiations: Vec<Instantiation>,
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
        self.parse_reader(reader, path)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn parse_reader<R: BufRead>(&mut self, reader: R, file_name: &str) -> Result<(), String> {
        let mut current_entity: Option<Entity> = None;
        let mut current_arch_entity_idx: Option<usize> = None;
        let mut in_port_block = false;
        let mut generate_stack: Vec<usize> = Vec::new();

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
                        file_name: file_name.to_string(),
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
                            // Find the raw text starting right after the colon
                            let colon_idx = port_part.find(':').unwrap();
                            let raw_type_str = &port_part[colon_idx + 1..].trim();

                            // To handle cases like "std_logic_vector(105 downto 0 );"
                            // we slice off the trailing ';' and ');' safely.
                            let mut type_string = raw_type_str.to_string();
                            if type_string.ends_with(';') {
                                type_string.pop();
                                type_string = type_string.trim().to_string();
                            }
                            // Only strip an ending ')' if it's NOT part of an array block!
                            // If the count of '(' equals the count of ')', stripping the ')' breaks the compile.
                            let open_parens = type_string.chars().filter(|c| *c == '(').count();
                            let close_parens = type_string.chars().filter(|c| *c == ')').count();
                            if type_string.ends_with(')') && close_parens > open_parens {
                                type_string.pop();
                            }

                            let type_part = type_string.trim();

                            let type_part_str = type_part.to_string();
                            let type_tokens: Vec<&str> = type_part_str.split_whitespace().collect();
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
                                    // Extract data_type exactly as it appears after the mode string
                                    let first_token = type_tokens[0];
                                    let idx = type_part_str.find(first_token).unwrap()
                                        + first_token.len();
                                    let mut data_type = type_part_str[idx..].trim().to_string();
                                    let mut range = None;

                                    // Extract explicit array blocks like: std_logic_vector(105 downto 0) natively mapping bounds
                                    if let Some(open_idx) = data_type.find('(') {
                                        if let Some(close_idx) = data_type.rfind(')') {
                                            if close_idx > open_idx {
                                                let range_str = data_type[open_idx + 1..close_idx]
                                                    .trim()
                                                    .to_string();
                                                if let Some(downto_idx) =
                                                    range_str.to_lowercase().find(" downto ")
                                                {
                                                    range = Some(Range {
                                                        left: range_str[..downto_idx]
                                                            .trim()
                                                            .to_string(),
                                                        right: range_str[downto_idx + 8..]
                                                            .trim()
                                                            .to_string(),
                                                        direction: Direction::Downto,
                                                    });
                                                } else if let Some(to_idx) =
                                                    range_str.to_lowercase().find(" to ")
                                                {
                                                    range = Some(Range {
                                                        left: range_str[..to_idx]
                                                            .trim()
                                                            .to_string(),
                                                        right: range_str[to_idx + 4..]
                                                            .trim()
                                                            .to_string(),
                                                        direction: Direction::To,
                                                    });
                                                }
                                                data_type =
                                                    data_type[..open_idx].trim().to_string();
                                            }
                                        }
                                    }

                                    let names: Vec<&str> = name_part.split(',').collect();
                                    for name in names {
                                        let clean_name = name.trim();
                                        if !clean_name.is_empty() {
                                            entity.ports.push(Port {
                                                name: clean_name.to_string(),
                                                mode: valid_mode.clone(),
                                                data_type: data_type.clone(),
                                                range: range.clone(),
                                                file_name: file_name.to_string(),
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
                    generate_stack.clear();
                } else if let Some(arch_idx) = current_arch_entity_idx {
                    // Inside architecture body

                    if line_str_no_comment.ends_with("generate")
                        || line_str_no_comment.ends_with("generate;")
                    {
                        if line_str_no_comment.starts_with("end generate")
                            || line_str_no_comment.contains(" end generate")
                        {
                            let _ = generate_stack.pop();
                        } else if line_str_no_comment.contains(" for ") {
                            let count = {
                                let ent = &self.entities[arch_idx];
                                parse_range_size(&raw_line_lower, Some(ent))
                            };
                            generate_stack.push(count);
                        }
                    } else if line_str_no_comment.contains(":") {
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

                                let mut multiplier: usize = generate_stack.iter().product();
                                if multiplier == 0 {
                                    multiplier = 1;
                                }

                                for _ in 0..multiplier {
                                    self.entities[arch_idx].instantiations.push(Instantiation {
                                        entity_name: name.clone(),
                                        ports: Vec::new(),
                                        file_name: file_name.to_string(),
                                        line: line_num,
                                    });
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
                if !all_instantiated.contains(&inst.entity_name.to_lowercase()) {
                    all_instantiated.push(inst.entity_name.to_lowercase());
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
            for (idx, root) in roots.iter().enumerate() {
                let mut path = Vec::new();
                let current_id = format!("{}", idx + 1);
                self.print_entity_tree(root, 0, &mut path, &current_id, None);
            }
        }
    }

    fn print_entity_tree<'a>(
        &'a self,
        entity: &'a Entity,
        depth: usize,
        path: &mut Vec<&'a str>,
        current_id: &str,
        instantiation_origin: Option<&Instantiation>,
    ) {
        let indent = " ".repeat(depth * 4);

        if path.contains(&entity.name.as_str()) {
            println!(
                "{}[ID: {}] Entity: {} (Defined: {}:{}) [Circular Dependency Detected]",
                indent, current_id, entity.name, entity.file_name, entity.line
            );
            return;
        }

        path.push(&entity.name);

        if let Some(inst) = instantiation_origin {
            println!(
                "{}[ID: {}] Entity: {} (Defined: {}:{} | Instantiated: {}:{})",
                indent,
                current_id,
                entity.name,
                entity.file_name,
                entity.line,
                inst.file_name,
                inst.line
            );
        } else {
            println!(
                "{}[ID: {}] Entity: {} (Defined: {}:{})",
                indent, current_id, entity.name, entity.file_name, entity.line
            );
        }

        let port_indent = " ".repeat(depth * 4 + 2);
        for (p_idx, port) in entity.ports.iter().enumerate() {
            if let Some(ref r) = port.range {
                let dir_str = match r.direction {
                    Direction::To => "to",
                    Direction::Downto => "downto",
                };
                println!(
                    "{}[ID: {}.p{}] Port: {} : {:?} {} [Range: {} {} {}] ({}:{})",
                    port_indent,
                    current_id,
                    p_idx + 1,
                    port.name,
                    port.mode,
                    port.data_type,
                    r.left,
                    dir_str,
                    r.right,
                    port.file_name,
                    port.line
                );
            } else {
                println!(
                    "{}[ID: {}.p{}] Port: {} : {:?} {} ({}:{})",
                    port_indent,
                    current_id,
                    p_idx + 1,
                    port.name,
                    port.mode,
                    port.data_type,
                    port.file_name,
                    port.line
                );
            }
        }

        for (inst_idx, inst) in entity.instantiations.iter().enumerate() {
            if let Some(child_entity) = self
                .entities
                .iter()
                .find(|e| e.name.eq_ignore_ascii_case(&inst.entity_name))
            {
                let child_id = format!("{}.{}", current_id, inst_idx + 1);
                self.print_entity_tree(child_entity, depth + 1, path, &child_id, Some(inst));
            } else {
                let child_indent = " ".repeat((depth + 1) * 4);
                println!(
                    "{}[ID: {}.{}] [External/Unknown Entity: {} (Instantiated: {}:{})]",
                    child_indent,
                    current_id,
                    inst_idx + 1,
                    inst.entity_name,
                    inst.file_name,
                    inst.line
                );
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
        project
            .parse_reader(vhdl_content.as_bytes(), "test.vhd")
            .unwrap();

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
        assert!(
            top_level
                .instantiations
                .iter()
                .any(|i| i.entity_name == "AND_Gate")
        );
        assert!(
            top_level
                .instantiations
                .iter()
                .any(|i| i.entity_name == "OR_Gate")
        );
    }

    #[test]
    fn test_duplicate_instantiations() {
        let vhdl_content = "
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity Basic_Gate is
    Port ( A : in  STD_LOGIC;
           Y : out STD_LOGIC);
end Basic_Gate;

entity Top_Level is
    Port ( In1 : in STD_LOGIC;
           Out1 : out STD_LOGIC);
end Top_Level;

architecture Structural of Top_Level is
begin
    inst_one: entity work.Basic_Gate port map (A => In1, Y => Out1);
    inst_two: entity work.Basic_Gate port map (A => In1, Y => Out1);
    
    gen_gates: for i in 0 to 1 generate
        inst_gen: entity work.Basic_Gate port map (A => In1, Y => Out1);
    end generate;
end Structural;
";
        let mut project = VhdlProject::new();
        project
            .parse_reader(vhdl_content.as_bytes(), "test.vhd")
            .unwrap();

        let top_level = project
            .entities
            .iter()
            .find(|e| e.name == "Top_Level")
            .unwrap();

        // 2 explicit maps + 1 generate loop spanning (0 to 1 = 2) "entity work.Basic_Gate" line => 4 mappings total
        assert_eq!(top_level.instantiations.len(), 4);
        assert_eq!(
            top_level
                .instantiations
                .iter()
                .filter(|&inst| inst.entity_name == "Basic_Gate")
                .count(),
            4
        );
    }

    #[test]
    fn test_range_generate() {
        let vhdl_content = "
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity Basic_Gate is
    Port ( A : in  STD_LOGIC;
           Y : out STD_LOGIC);
end Basic_Gate;

entity Top_Level is
    Port ( Ptr_Array : in STD_LOGIC_VECTOR(7 downto 0);
           Out1 : out STD_LOGIC);
end Top_Level;

architecture Structural of Top_Level is
begin
    gen_gates: for i in Ptr_Array'range generate
        inst_gen: entity work.Basic_Gate port map (A => Ptr_Array(i), Y => Out1);
    end generate;
end Structural;
";
        let mut project = VhdlProject::new();
        project
            .parse_reader(vhdl_content.as_bytes(), "test.vhd")
            .unwrap();

        let top_level = project
            .entities
            .iter()
            .find(|e| e.name == "Top_Level")
            .unwrap();

        // Ptr_Array is (7 downto 0) -> abs(7 - 0) + 1 = 8 mappings
        assert_eq!(top_level.instantiations.len(), 8);
        assert_eq!(
            top_level
                .instantiations
                .iter()
                .filter(|&inst| inst.entity_name == "Basic_Gate")
                .count(),
            8
        );
    }
}
