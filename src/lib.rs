use serde::Serialize;
use std::collections::HashMap;
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

pub fn calculate_bits(
    data_type: &str,
    range: Option<&Range>,
    entity: Option<&Entity>,
) -> Option<usize> {
    let dt = data_type.to_lowercase();
    if dt == "std_logic" || dt == "bit" {
        return Some(1);
    }

    if let Some(r) = range {
        let dir_str = match r.direction {
            Direction::To => "to",
            Direction::Downto => "downto",
        };
        let count = parse_range_size(&format!("{} {} {}", r.left, dir_str, r.right), entity);

        let mut bits_per_element = 1;
        if dt.contains("usb_byte_array_t")
            || dt.contains("usb_byte_t")
            || dt.contains("usb_ep_input_signals_t")
            || dt.contains("usb_ep_output_signals_t")
        {
            bits_per_element = 8;
        } else if dt.contains("usb_dev_addr_t") {
            bits_per_element = 7;
        }

        return Some(count * bits_per_element);
    }

    None
}

pub fn parse_range_size(range_expr: &str, entity: Option<&Entity>) -> usize {
    let expr = range_expr.to_lowercase();

    // Check for 'range
    if let Some(idx) = expr.find("'range") {
        let before_range = &expr[..idx];
        let tokens: Vec<&str> = before_range.split_whitespace().collect();
        if let Some(&signal_name) = tokens.last()
            && let Some(ent) = entity
        {
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
                    return parse_range_size(&format!("{} {} {}", r.left, dir_str, r.right), None);
                } else {
                    return parse_range_size(&port.data_type, None);
                }
            }
        }
        return 1; // Fallback
    }

    let mut parts = Vec::new();
    if let Some(idx) = expr.find("downto") {
        parts.push(&expr[..idx]);
        parts.push(&expr[idx + 6..]);
    } else if let Some(idx) = expr.find("to") {
        parts.push(&expr[..idx]);
        parts.push(&expr[idx + 2..]);
    }

    if parts.len() >= 2 {
        let left = extract_last_number(parts[0]);
        let right = extract_first_number(parts[1]);
        if let (Some(l), Some(r)) = (left, right) {
            return (l - r).unsigned_abs() as usize + 1;
        }
    }

    1
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum PortMode {
    In,
    Out,
    InOut,
    Buffer,
    Linkage,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Direction {
    To,
    Downto,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Range {
    pub left: String,
    pub direction: Direction,
    pub right: String,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Port {
    pub name: String,
    pub mode: PortMode,
    pub data_type: String,    // E.g., std_logic, std_logic_vector
    pub range: Option<Range>, // E.g., 7 downto 0
    pub file_name: String,
    pub line: usize,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Instantiation {
    pub entity_name: String,
    pub ports: Vec<Port>,
    #[serde(skip_serializing)]
    pub port_map: HashMap<String, String>,
    pub file_name: String,
    pub line: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Identifier(String),    // Matches words, keywords (e.g., "entity", "std_logic")
    Number(String),        // Matches numbers (e.g., "105", "0")
    StringLiteral(String), // Matches string literals
    CharLiteral(char),     // Matches character literals (e.g. '0')
    Symbol(String),        // Matches operators/symbols: "(", ")", ":", ";", ",", "=>", etc.
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Entity {
    pub name: String,
    pub ports: Vec<Port>,
    pub signals: Vec<Port>,
    pub file_name: String,
    pub line: usize,
    pub instantiations: Vec<Instantiation>,
}

#[derive(Debug, Serialize)]
pub struct EntityNode {
    pub id: String,
    #[serde(flatten)]
    pub entity: Entity,
    pub children: Vec<EntityNode>,
}

#[derive(Debug, Serialize)]
pub struct VhdlProject {
    pub entities: Vec<Entity>,
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

        // Pass the entire VHDL document securely to the Tokenizer
        let tokens = Self::tokenize(&content);

        // Process tokens correctly mapping the state machine
        self.parse_tokens(&tokens, file_name);

        Ok(())
    }

    pub fn export_json_tree(&self) -> Vec<EntityNode> {
        let mut all_instantiated = Vec::new();
        for entity in &self.entities {
            for inst in &entity.instantiations {
                if !all_instantiated.contains(&inst.entity_name.to_lowercase()) {
                    all_instantiated.push(inst.entity_name.to_lowercase());
                }
            }
        }

        let mut root_nodes = Vec::new();
        for (_idx, entity) in self.entities.iter().enumerate() {
            if !all_instantiated.contains(&entity.name.to_lowercase()) {
                let id = format!("{}", root_nodes.len() + 1);
                root_nodes.push(self.build_entity_node(entity, &id, &mut Vec::new()));
            }
        }
        root_nodes
    }

    fn build_entity_node<'a>(
        &'a self,
        entity: &'a Entity,
        current_id: &str,
        path: &mut Vec<&'a str>,
    ) -> EntityNode {
        if path.contains(&entity.name.as_str()) {
            return EntityNode {
                id: current_id.to_string(),
                entity: entity.clone(),
                children: Vec::new(),
            };
        }

        path.push(&entity.name);

        let mut children = Vec::new();
        for (inst_idx, child_inst) in entity.instantiations.iter().enumerate() {
            if let Some(child_ent) = self
                .entities
                .iter()
                .find(|e| e.name.eq_ignore_ascii_case(&child_inst.entity_name))
            {
                let child_id = format!("{}.{}", current_id, inst_idx + 1);
                children.push(self.build_entity_node(child_ent, &child_id, path));
            }
        }

        path.pop();

        EntityNode {
            id: current_id.to_string(),
            entity: entity.clone(),
            children,
        }
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
                self.print_entity_tree(root, 0, &mut path, &current_id, None, None);
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
        parent_entity: Option<&'a Entity>,
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
            let mut resolved_range = port.range.clone();
            let mut mapped_msg = String::new();

            if resolved_range.is_none()
                && let Some(inst) = instantiation_origin
            {
                let mut matched_key = None;
                let mut matched_val = None;

                for (k, v) in &inst.port_map {
                    if k.eq_ignore_ascii_case(&port.name)
                        || k.to_lowercase()
                            .starts_with(&format!("{}(", port.name.to_lowercase()))
                    {
                        matched_key = Some(k.clone());
                        matched_val = Some(v.clone());
                        break;
                    }
                }

                if let Some(mapped_val) = matched_val {
                    let k = matched_key.unwrap();
                    let mut display_val = mapped_val.clone();

                    if !k.eq_ignore_ascii_case(&port.name) {
                        display_val = format!("{} via {}", mapped_val, k);
                    }

                    let mut base_sig_name = mapped_val.clone();
                    if let Some(idx) = base_sig_name.find('(') {
                        base_sig_name = base_sig_name[..idx].trim().to_string();
                    }

                    // Determine explicit range if the mapping key was sliced, e.g. INPUT(3 downto 0) or INPUT(0)
                    if let Some(slice_start) = k.find('(')
                        && let Some(slice_end) = k.rfind(')')
                    {
                        let slice_val = &k[slice_start + 1..slice_end];
                        let dir = if slice_val.contains("downto") {
                            Direction::Downto
                        } else {
                            Direction::To
                        };
                        let parts: Vec<&str> = if dir == Direction::Downto {
                            slice_val.split("downto").collect()
                        } else {
                            slice_val.split("to").collect()
                        };
                        if parts.len() == 2 {
                            resolved_range = Some(Range {
                                left: parts[0].trim().to_string(),
                                right: parts[1].trim().to_string(),
                                direction: dir,
                            });
                        } else {
                            resolved_range = Some(Range {
                                left: slice_val.trim().to_string(),
                                right: slice_val.trim().to_string(),
                                direction: dir,
                            });
                        }
                    }

                    // Look up in parent
                    mapped_msg = format!(" [Mapped to: {}]", display_val);
                    if resolved_range.is_none()
                        && let Some(parent) = parent_entity
                    {
                        if let Some(p_port) = parent
                            .ports
                            .iter()
                            .find(|p| p.name.eq_ignore_ascii_case(&base_sig_name))
                        {
                            resolved_range = p_port.range.clone();
                            if p_port.range.is_none()
                                && let Some(start) = p_port.data_type.find('(')
                                && let Some(end) = p_port.data_type.rfind(')')
                            {
                                mapped_msg = format!(
                                    " [Mapped to: {} -> Range: {}]",
                                    display_val,
                                    &p_port.data_type[start + 1..end].trim()
                                );
                                let r_str = &p_port.data_type[start + 1..end].trim();
                                let dir = if r_str.contains("downto") {
                                    Direction::Downto
                                } else {
                                    Direction::To
                                };
                                let parts: Vec<&str> = if dir == Direction::Downto {
                                    r_str.split("downto").collect()
                                } else {
                                    r_str.split("to").collect()
                                };
                                if parts.len() == 2 {
                                    resolved_range = Some(Range {
                                        left: parts[0].trim().to_string(),
                                        right: parts[1].trim().to_string(),
                                        direction: dir,
                                    });
                                }
                            }
                        } else if let Some(p_sig) = parent
                            .signals
                            .iter()
                            .find(|s| s.name.eq_ignore_ascii_case(&base_sig_name))
                            && let Some(start) = p_sig.data_type.find('(')
                            && let Some(end) = p_sig.data_type.rfind(')')
                        {
                            mapped_msg = format!(
                                " [Mapped to: {} -> Range: {}]",
                                display_val,
                                &p_sig.data_type[start + 1..end].trim()
                            );
                            let r_str = &p_sig.data_type[start + 1..end].trim();
                            let dir = if r_str.contains("downto") {
                                Direction::Downto
                            } else {
                                Direction::To
                            };
                            let parts: Vec<&str> = if dir == Direction::Downto {
                                r_str.split("downto").collect()
                            } else {
                                r_str.split("to").collect()
                            };
                            if parts.len() == 2 {
                                resolved_range = Some(Range {
                                    left: parts[0].trim().to_string(),
                                    direction: dir,
                                    right: parts[1].trim().to_string(),
                                });
                            }
                        }
                    }
                }
            }

            let mut size_msg = String::new();
            if let Some(size) =
                calculate_bits(&port.data_type, resolved_range.as_ref(), parent_entity)
            {
                size_msg = format!(" [Size: {} bits]", size);
            }

            if let Some(ref r) = resolved_range {
                let dir_str = match r.direction {
                    Direction::To => "to",
                    Direction::Downto => "downto",
                };
                println!(
                    "{}[ID: {}.p{}] Port: {} : {:?} {}{} [Range: {} {} {}]{} ({}:{})",
                    port_indent,
                    current_id,
                    p_idx + 1,
                    port.name,
                    port.mode,
                    port.data_type,
                    mapped_msg,
                    r.left,
                    dir_str,
                    r.right,
                    size_msg,
                    port.file_name,
                    port.line
                );
            } else {
                println!(
                    "{}[ID: {}.p{}] Port: {} : {:?} {}{}{} ({}:{})",
                    port_indent,
                    current_id,
                    p_idx + 1,
                    port.name,
                    port.mode,
                    port.data_type,
                    mapped_msg,
                    size_msg,
                    port.file_name,
                    port.line
                );
            }
        }

        let sig_indent = " ".repeat(depth * 4 + 2);
        for (s_idx, sig) in entity.signals.iter().enumerate() {
            let mut size_msg = String::new();
            if let Some(size) = calculate_bits(&sig.data_type, sig.range.as_ref(), Some(entity)) {
                size_msg = format!(" [Size: {} bits]", size);
            }

            if let Some(ref r) = sig.range {
                let dir_str = match r.direction {
                    Direction::To => "to",
                    Direction::Downto => "downto",
                };
                println!(
                    "{}[ID: {}.s{}] Signal: {} : {} [Range: {} {} {}]{} ({}:{})",
                    sig_indent,
                    current_id,
                    s_idx + 1,
                    sig.name,
                    sig.data_type,
                    r.left,
                    dir_str,
                    r.right,
                    size_msg,
                    sig.file_name,
                    sig.line
                );
            } else {
                println!(
                    "{}[ID: {}.s{}] Signal: {} : {}{} ({}:{})",
                    sig_indent,
                    current_id,
                    s_idx + 1,
                    sig.name,
                    sig.data_type,
                    size_msg,
                    sig.file_name,
                    sig.line
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
                self.print_entity_tree(
                    child_entity,
                    depth + 1,
                    path,
                    &child_id,
                    Some(inst),
                    Some(entity),
                );
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

    pub fn tokenize(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut line = 1;
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                if c == '\n' {
                    line += 1;
                }
                chars.next();
                continue;
            }

            match c {
                '-' => {
                    chars.next();
                    if chars.peek() == Some(&'-') {
                        // VHDL Comment! Skip to newline
                        while let Some(&nc) = chars.peek() {
                            if nc == '\n' {
                                break;
                            }
                            chars.next();
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Symbol("-".to_string()),
                            line,
                        });
                    }
                }
                '=' => {
                    chars.next();
                    if chars.peek() == Some(&'>') {
                        chars.next();
                        tokens.push(Token {
                            token_type: TokenType::Symbol("=>".to_string()),
                            line,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Symbol("=".to_string()),
                            line,
                        });
                    }
                }
                ':' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {
                            token_type: TokenType::Symbol(":=".to_string()),
                            line,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Symbol(":".to_string()),
                            line,
                        });
                    }
                }
                '/' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {
                            token_type: TokenType::Symbol("/=".to_string()),
                            line,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Symbol("/".to_string()),
                            line,
                        });
                    }
                }
                '<' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {
                            token_type: TokenType::Symbol("<=".to_string()),
                            line,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Symbol("<".to_string()),
                            line,
                        });
                    }
                }
                '>' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {
                            token_type: TokenType::Symbol(">=".to_string()),
                            line,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Symbol(">".to_string()),
                            line,
                        });
                    }
                }
                '"' => {
                    chars.next();
                    let mut s = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc == '"' {
                            chars.next(); // Consume closing quote
                            break;
                        } else if nc == '\n' {
                            line += 1;
                        }
                        s.push(nc);
                        chars.next();
                    }
                    tokens.push(Token {
                        token_type: TokenType::StringLiteral(s),
                        line,
                    });
                }
                '\'' => {
                    chars.next(); // consume '\''
                    if let Some(&nc) = chars.peek() {
                        if nc.is_alphabetic() {
                            // Attribute! E.g., 'range or 'high
                            let mut ident = String::new();
                            ident.push('\'');
                            while let Some(&cc) = chars.peek() {
                                if cc.is_alphanumeric() || cc == '_' {
                                    ident.push(cc);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            tokens.push(Token {
                                token_type: TokenType::Identifier(ident),
                                line,
                            });
                        } else {
                            // E.g. bit string '1'
                            let val = nc;
                            chars.next();
                            if chars.peek() == Some(&'\'') {
                                chars.next();
                                tokens.push(Token {
                                    token_type: TokenType::CharLiteral(val),
                                    line,
                                });
                            } else {
                                tokens.push(Token {
                                    token_type: TokenType::Symbol("'".to_string()),
                                    line,
                                });
                            }
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Symbol("'".to_string()),
                            line,
                        });
                    }
                }
                '(' | ')' | ';' | ',' | '+' | '*' | '&' | '|' | '.' => {
                    tokens.push(Token {
                        token_type: TokenType::Symbol(c.to_string()),
                        line,
                    });
                    chars.next();
                }
                _ if c.is_alphabetic() => {
                    let mut ident = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc.is_alphanumeric() || nc == '_' || nc == '.' {
                            ident.push(nc);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token {
                        token_type: TokenType::Identifier(ident),
                        line,
                    });
                }
                _ if c.is_ascii_digit() => {
                    let mut num = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc.is_ascii_digit()
                            || nc == '.'
                            || nc == '_'
                            || nc.is_alphabetic()
                            || nc == '#'
                        {
                            // E.g. sizes like 48MHz or 16#FF#
                            num.push(nc);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token {
                        token_type: TokenType::Number(num),
                        line,
                    });
                }
                _ => {
                    tokens.push(Token {
                        token_type: TokenType::Symbol(c.to_string()),
                        line,
                    });
                    chars.next();
                }
            }
        }
        tokens
    }

    pub fn parse_tokens(&mut self, tokens: &[Token], file_name: &str) {
        let mut idx = 0;
        let mut current_entity: Option<Entity> = None;
        let mut current_arch_entity_idx: Option<usize> = None;
        let mut current_arch_name: Option<String> = None;
        let mut generate_stack: Vec<usize> = Vec::new();

        while idx < tokens.len() {
            let t = &tokens[idx];

            match &t.token_type {
                // Wait for entity declaration
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("entity") => {
                    idx += 1;
                    if let Some(name_token) = tokens.get(idx)
                        && let TokenType::Identifier(name) = &name_token.token_type
                    {
                        idx += 1;
                        if let Some(is_token) = tokens.get(idx)
                            && let TokenType::Identifier(is_id) = &is_token.token_type
                            && is_id.eq_ignore_ascii_case("is")
                        {
                            current_entity = Some(Entity {
                                name: name.clone(),
                                ports: Vec::new(),
                                signals: Vec::new(),
                                file_name: file_name.to_string(),
                                line: t.line,
                                instantiations: Vec::new(),
                            });
                        }
                    }
                }

                // End Entity OR End Architecture
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("end") => {
                    idx += 1;
                    if let Some(next_token) = tokens.get(idx) {
                        if let TokenType::Identifier(next_id) = &next_token.token_type {
                            if next_id.eq_ignore_ascii_case("entity")
                                || (current_entity.is_some()
                                    && next_id.eq_ignore_ascii_case(
                                        &current_entity.as_ref().unwrap().name,
                                    ))
                            {
                                if let Some(ent) = current_entity.take() {
                                    self.entities.push(ent);
                                }
                                idx += 1;
                            } else if next_id.eq_ignore_ascii_case("architecture")
                                || next_id.eq_ignore_ascii_case("behavioral")
                                || next_id.eq_ignore_ascii_case("structural")
                                || next_id.eq_ignore_ascii_case("rtl")
                                || (current_arch_name.is_some()
                                    && next_id
                                        .eq_ignore_ascii_case(current_arch_name.as_ref().unwrap()))
                            {
                                current_arch_entity_idx = None;
                                current_arch_name = None;
                                generate_stack.clear();
                                idx += 1;
                            } else if next_id.eq_ignore_ascii_case("generate") {
                                // end generate
                                let _ = generate_stack.pop();
                                idx += 1;
                            }
                        } else if let TokenType::Symbol(sym) = &next_token.token_type
                            && sym == ";"
                        {
                            // Simple `end;` -> usually ends entity if currently parsing one
                            if let Some(ent) = current_entity.take() {
                                self.entities.push(ent);
                            }
                        }
                    }
                }

                // Skip generic declarations inside entities and components
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("generic") => {
                    idx += 1;
                    if let Some(paren_token) = tokens.get(idx)
                        && let TokenType::Symbol(sym) = &paren_token.token_type
                        && sym == "("
                    {
                        idx += 1;
                        let mut paren_depth = 1;
                        while idx < tokens.len() && paren_depth > 0 {
                            if let TokenType::Symbol(s) = &tokens[idx].token_type {
                                if s == "(" {
                                    paren_depth += 1;
                                } else if s == ")" {
                                    paren_depth -= 1;
                                }
                            }
                            idx += 1;
                        }
                    }
                    continue;
                }

                // Port parsing inside Entity
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("port") => {
                    if current_entity.is_some() && current_arch_entity_idx.is_none() {
                        idx += 1;
                        if let Some(paren_token) = tokens.get(idx)
                            && let TokenType::Symbol(sym) = &paren_token.token_type
                            && sym == "("
                        {
                            idx += 1;
                            // Parse ports until matched closing parenthesis
                            let mut paren_depth = 1;
                            let mut port_names = Vec::new();

                            while idx < tokens.len() && paren_depth > 0 {
                                // Collect port names separated by commas
                                port_names.clear();
                                while idx < tokens.len() {
                                    if let TokenType::Identifier(p_name) = &tokens[idx].token_type {
                                        port_names.push((p_name.clone(), tokens[idx].line));
                                        idx += 1;
                                        if let TokenType::Symbol(s) = &tokens[idx].token_type {
                                            if s == "," {
                                                idx += 1;
                                                continue;
                                            } else if s == ":" {
                                                idx += 1; // Consume colon
                                                break;
                                            }
                                        }
                                    } else {
                                        idx += 1;
                                    }
                                }

                                // Parse Direction (in, out, inout, etc.)
                                let mut mode = None;
                                if let TokenType::Identifier(m) = &tokens[idx].token_type {
                                    mode = match m.to_lowercase().as_str() {
                                        "in" => Some(PortMode::In),
                                        "out" => Some(PortMode::Out),
                                        "inout" => Some(PortMode::InOut),
                                        "buffer" => Some(PortMode::Buffer),
                                        "linkage" => Some(PortMode::Linkage),
                                        _ => None,
                                    };
                                    if mode.is_some() {
                                        idx += 1;
                                    }
                                }

                                // Parse Data Type
                                let mut data_type = String::new();
                                let mut range: Option<Range> = None;

                                while idx < tokens.len() {
                                    match &tokens[idx].token_type {
                                        TokenType::Symbol(s) if s == ";" => {
                                            idx += 1; // Consume ;
                                            break; // End of this port declaration
                                        }
                                        TokenType::Symbol(s) if s == ":=" => {
                                            // Consume initialization completely
                                            while idx < tokens.len() {
                                                if let TokenType::Symbol(end_s) =
                                                    &tokens[idx].token_type
                                                    && (end_s == ";" || end_s == ")")
                                                {
                                                    break;
                                                }
                                                idx += 1;
                                            }
                                            continue;
                                        }
                                        TokenType::Symbol(s) if s == ")" => {
                                            paren_depth -= 1;
                                            if paren_depth == 0 {
                                                idx += 1; // Consume )
                                                break; // End of entire port block
                                            }
                                            data_type.push_str(s);
                                        }
                                        TokenType::Symbol(s) if s == "(" => {
                                            paren_depth += 1;

                                            // This might be the start of a range block e.g., (105 downto 0)
                                            let mut range_tokens = Vec::new();
                                            idx += 1;
                                            let mut inner_depth = 1;
                                            data_type.push('(');
                                            while idx < tokens.len() && inner_depth > 0 {
                                                match &tokens[idx].token_type {
                                                    TokenType::Symbol(inner_s)
                                                        if inner_s == "(" =>
                                                    {
                                                        inner_depth += 1;
                                                    }
                                                    TokenType::Symbol(inner_s)
                                                        if inner_s == ")" =>
                                                    {
                                                        inner_depth -= 1;
                                                    }
                                                    _ => {}
                                                }

                                                if inner_depth > 0 {
                                                    range_tokens.push(tokens[idx].clone());
                                                    match &tokens[idx].token_type {
                                                        TokenType::Identifier(i)
                                                        | TokenType::Number(i)
                                                        | TokenType::Symbol(i)
                                                        | TokenType::StringLiteral(i) => {
                                                            data_type.push_str(i);
                                                            if !matches!(
                                                                &tokens[idx].token_type,
                                                                TokenType::Symbol(_)
                                                            ) {
                                                                data_type.push(' ');
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                } else {
                                                    data_type = data_type.trim_end().to_string(); // remove trailing space before closing paren
                                                    data_type.push(')');
                                                }
                                                idx += 1;
                                            }
                                            paren_depth -= 1; // We matched the closing paren of the range

                                            // Try reconstructing the range
                                            let mut r_left = String::new();
                                            let mut r_right = String::new();
                                            let mut r_dir = None;
                                            let mut hit_dir = false;

                                            for rt in &range_tokens {
                                                match &rt.token_type {
                                                    TokenType::Identifier(dir)
                                                        if dir.eq_ignore_ascii_case("downto") =>
                                                    {
                                                        r_dir = Some(Direction::Downto);
                                                        hit_dir = true;
                                                    }
                                                    TokenType::Identifier(dir)
                                                        if dir.eq_ignore_ascii_case("to") =>
                                                    {
                                                        r_dir = Some(Direction::To);
                                                        hit_dir = true;
                                                    }
                                                    TokenType::Identifier(i)
                                                    | TokenType::Number(i)
                                                    | TokenType::Symbol(i)
                                                    | TokenType::StringLiteral(i) => {
                                                        if hit_dir {
                                                            r_right.push_str(i);
                                                            r_right.push(' ');
                                                        } else {
                                                            r_left.push_str(i);
                                                            r_left.push(' ');
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }

                                            if let Some(d) = r_dir {
                                                range = Some(Range {
                                                    left: r_left.trim().to_string(),
                                                    direction: d,
                                                    right: r_right.trim().to_string(),
                                                });
                                            } else {
                                                // It was just a regular type parameter like std_logic_vector(0)
                                                data_type.push('(');
                                                for rt in range_tokens {
                                                    if let TokenType::Identifier(i)
                                                    | TokenType::Number(i)
                                                    | TokenType::Symbol(i) = rt.token_type
                                                    {
                                                        data_type.push_str(&i);
                                                    }
                                                }
                                                data_type.push(')');
                                            }
                                            continue;
                                        }
                                        TokenType::Identifier(i)
                                        | TokenType::Number(i)
                                        | TokenType::Symbol(i) => {
                                            data_type.push_str(i);
                                            data_type.push(' ');
                                        }
                                        _ => {}
                                    }
                                    idx += 1;
                                }

                                // Apply ports if valid
                                if let Some(valid_mode) = mode {
                                    for (p_name, p_line) in &port_names {
                                        if let Some(ent) = &mut current_entity {
                                            ent.ports.push(Port {
                                                name: p_name.clone(),
                                                mode: valid_mode.clone(),
                                                data_type: data_type.trim().to_string(),
                                                range: range.clone(),
                                                file_name: file_name.to_string(),
                                                line: *p_line,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Architecture Start
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("architecture") => {
                    idx += 1;
                    if let Some(name_token) = tokens.get(idx)
                        && let TokenType::Identifier(arch_name) = &name_token.token_type
                    {
                        current_arch_name = Some(arch_name.clone());
                        // Skip name
                        idx += 1;
                        if let Some(of_token) = tokens.get(idx)
                            && let TokenType::Identifier(of_id) = &of_token.token_type
                            && of_id.eq_ignore_ascii_case("of")
                        {
                            idx += 1;
                            if let Some(target_token) = tokens.get(idx)
                                && let TokenType::Identifier(target_name) = &target_token.token_type
                            {
                                current_arch_entity_idx = self
                                    .entities
                                    .iter()
                                    .position(|e| e.name.eq_ignore_ascii_case(target_name));
                            }
                        }
                    }
                }

                // Parse Signals inside Architecture
                TokenType::Identifier(id)
                    if id.eq_ignore_ascii_case("signal") || id.eq_ignore_ascii_case("constant") =>
                {
                    if let Some(arch_idx) = current_arch_entity_idx {
                        idx += 1;
                        let mut sig_names = Vec::new();

                        // Collect names until ':'
                        while idx < tokens.len() {
                            if let TokenType::Identifier(sig) = &tokens[idx].token_type {
                                sig_names.push((sig.clone(), tokens[idx].line));
                            } else if let TokenType::Symbol(sym) = &tokens[idx].token_type
                                && sym == ":"
                            {
                                idx += 1;
                                break;
                            }
                            idx += 1;
                        }

                        // Collect type and range until ';' or ':='
                        let mut data_type = String::new();
                        let mut range: Option<Range> = None;

                        while idx < tokens.len() {
                            match &tokens[idx].token_type {
                                TokenType::Symbol(s) if s == ";" => {
                                    break;
                                }
                                TokenType::Symbol(s) if s == ":=" => {
                                    // Consume initialization completely
                                    while idx < tokens.len() {
                                        if let TokenType::Symbol(end_s) = &tokens[idx].token_type
                                            && end_s == ";"
                                        {
                                            break;
                                        }
                                        idx += 1;
                                    }
                                    break;
                                }
                                TokenType::Symbol(s) if s == "(" => {
                                    // Range parsing block same as ports
                                    let mut range_tokens = Vec::new();
                                    idx += 1;
                                    let mut inner_depth = 1;
                                    data_type.push('(');
                                    while idx < tokens.len() && inner_depth > 0 {
                                        match &tokens[idx].token_type {
                                            TokenType::Symbol(inner_s) if inner_s == "(" => {
                                                inner_depth += 1;
                                            }
                                            TokenType::Symbol(inner_s) if inner_s == ")" => {
                                                inner_depth -= 1;
                                            }
                                            _ => {}
                                        }

                                        if inner_depth > 0 {
                                            range_tokens.push(tokens[idx].clone());
                                            match &tokens[idx].token_type {
                                                TokenType::Identifier(i)
                                                | TokenType::Number(i)
                                                | TokenType::Symbol(i)
                                                | TokenType::StringLiteral(i) => {
                                                    data_type.push_str(i);
                                                    if !matches!(
                                                        &tokens[idx].token_type,
                                                        TokenType::Symbol(_)
                                                    ) {
                                                        data_type.push(' ');
                                                    }
                                                }
                                                _ => {}
                                            }
                                        } else {
                                            data_type = data_type.trim_end().to_string(); // remove trailing space before closing paren
                                            data_type.push(')');
                                        }
                                        idx += 1;
                                    }
                                    let mut r_left = String::new();
                                    let mut r_right = String::new();
                                    let mut r_dir = None;
                                    let mut hit_dir = false;

                                    for rt in &range_tokens {
                                        match &rt.token_type {
                                            TokenType::Identifier(dir)
                                                if dir.eq_ignore_ascii_case("downto") =>
                                            {
                                                r_dir = Some(Direction::Downto);
                                                hit_dir = true;
                                            }
                                            TokenType::Identifier(dir)
                                                if dir.eq_ignore_ascii_case("to") =>
                                            {
                                                r_dir = Some(Direction::To);
                                                hit_dir = true;
                                            }
                                            TokenType::Identifier(i)
                                            | TokenType::Number(i)
                                            | TokenType::Symbol(i)
                                            | TokenType::StringLiteral(i) => {
                                                if hit_dir {
                                                    r_right.push_str(i);
                                                    r_right.push(' ');
                                                } else {
                                                    r_left.push_str(i);
                                                    r_left.push(' ');
                                                }
                                            }
                                            _ => {}
                                        }
                                    }

                                    if let Some(d) = r_dir {
                                        range = Some(Range {
                                            left: r_left.trim().to_string(),
                                            right: r_right.trim().to_string(),
                                            direction: d,
                                        });
                                    } else {
                                        // It was just a regular type parameter like std_logic_vector(0)
                                        data_type.push('(');
                                        for rt in range_tokens {
                                            if let TokenType::Identifier(i)
                                            | TokenType::Number(i)
                                            | TokenType::Symbol(i) = rt.token_type
                                            {
                                                data_type.push_str(&i);
                                            }
                                        }
                                        data_type.push(')');
                                    }
                                    continue;
                                }
                                TokenType::Identifier(i)
                                | TokenType::Number(i)
                                | TokenType::StringLiteral(i)
                                | TokenType::Symbol(i) => {
                                    data_type.push_str(i);
                                    if !matches!(&tokens[idx].token_type, TokenType::Symbol(_)) {
                                        data_type.push(' ');
                                    }
                                }
                                _ => {}
                            }
                            idx += 1;
                        }

                        for (s_name, s_line) in sig_names {
                            self.entities[arch_idx].signals.push(Port {
                                name: s_name,
                                mode: PortMode::In, // dummy mode
                                data_type: data_type.trim().to_string(),
                                range: range.clone(),
                                file_name: file_name.to_string(),
                                line: s_line,
                            });
                        }
                    }
                }

                // For logic tracking generate sizes
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("for") => {
                    if let Some(arch_idx) = current_arch_entity_idx {
                        // Skip variable name
                        idx += 2;
                        if let Some(in_t) = tokens.get(idx)
                            && let TokenType::Identifier(in_id) = &in_t.token_type
                            && in_id.eq_ignore_ascii_case("in")
                        {
                            idx += 1;
                            // Parse until `generate`
                            let mut range_str = String::new();
                            while idx < tokens.len() {
                                if let TokenType::Identifier(gen_id) = &tokens[idx].token_type
                                    && gen_id.eq_ignore_ascii_case("generate")
                                {
                                    let count = {
                                        let ent = &self.entities[arch_idx];
                                        parse_range_size(&range_str, Some(ent))
                                    };
                                    generate_stack.push(count);
                                    break;
                                }
                                if let TokenType::Identifier(i)
                                | TokenType::Number(i)
                                | TokenType::Symbol(i)
                                | TokenType::StringLiteral(i) = &tokens[idx].token_type
                                {
                                    if range_str.is_empty() {
                                        range_str.push_str(i);
                                    } else {
                                        range_str.push(' ');
                                        range_str.push_str(i);
                                    }
                                }
                                idx += 1;
                            }
                        }
                    }
                }

                // Component Instantiation Mapping
                TokenType::Symbol(sym) if sym == ":" => {
                    if let Some(arch_idx) = current_arch_entity_idx {
                        let start_idx = idx;
                        // Look backwards to find the instance name
                        let mut _inst_name = String::new();
                        let inst_line = t.line;
                        if idx > 0
                            && let TokenType::Identifier(prev_id) = &tokens[idx - 1].token_type
                        {
                            _inst_name = prev_id.clone();
                        }

                        // Look forwards to find what we are instantiating "entity work.X" or "Component_Name"
                        idx += 1;
                        let mut target_name = String::new();
                        if let Some(next_t) = tokens.get(idx)
                            && let TokenType::Identifier(nid) = &next_t.token_type
                        {
                            if nid.eq_ignore_ascii_case("entity") {
                                idx += 1; // get the actual entity name
                                if let Some(tgt_t) = tokens.get(idx)
                                    && let TokenType::Identifier(t_id) = &tgt_t.token_type
                                {
                                    // Extract name stripping 'work.' prefix if exists
                                    let mut t_name = t_id.clone();
                                    if t_name.to_lowercase().starts_with("work.") {
                                        t_name = t_name[5..].to_string();
                                    }
                                    target_name = t_name;
                                }
                            } else {
                                // Normally just "Component_Name"
                                target_name = nid.clone();
                                // Ensure it handles generic/port map correctly
                                idx += 1;
                                if let Some(follow_t) = tokens.get(idx) {
                                    if let TokenType::Identifier(fid) = &follow_t.token_type {
                                        if fid.eq_ignore_ascii_case("generic")
                                            || fid.eq_ignore_ascii_case("port")
                                        {
                                            // Valid instantiation
                                        } else {
                                            target_name.clear(); // Invalid structure
                                        }
                                    } else {
                                        target_name.clear(); // Invalid structure
                                    }
                                } else {
                                    target_name.clear(); // Invalid structure
                                }
                            }
                        }

                        if !target_name.is_empty() {
                            let mut port_map = HashMap::new();
                            let mut local_idx = idx;
                            let mut inside_map = false;
                            while local_idx < tokens.len() {
                                match &tokens[local_idx].token_type {
                                    TokenType::Identifier(id)
                                        if id.eq_ignore_ascii_case("port")
                                            || id.eq_ignore_ascii_case("generic") =>
                                    {
                                        if let Some(next_t) = tokens.get(local_idx + 1)
                                            && let TokenType::Identifier(mid) = &next_t.token_type
                                            && mid.eq_ignore_ascii_case("map")
                                        {
                                            inside_map = true;
                                            local_idx += 2; // skip map
                                            if let Some(TokenType::Symbol(s)) =
                                                tokens.get(local_idx).map(|t| &t.token_type)
                                                && s == "("
                                            {
                                                local_idx += 1;
                                            }
                                            continue;
                                        }
                                    }
                                    TokenType::Symbol(s) if s == ";" => {
                                        break; // End of instantiation
                                    }
                                    TokenType::Symbol(s) if s == "=>" && inside_map => {
                                        // Found mapping
                                        let mut inner_port = String::new();
                                        let mut rev_idx = local_idx;
                                        let mut rev_paren_depth = 0;
                                        while rev_idx > 0 {
                                            rev_idx -= 1;
                                            match &tokens[rev_idx].token_type {
                                                TokenType::Symbol(sym) if sym == ")" => {
                                                    rev_paren_depth += 1;
                                                    inner_port.insert_str(0, sym);
                                                }
                                                TokenType::Symbol(sym) if sym == "(" => {
                                                    if rev_paren_depth > 0 {
                                                        rev_paren_depth -= 1;
                                                        inner_port.insert_str(0, sym);
                                                    } else {
                                                        break; // We hit the opening paren of the overall port map list
                                                    }
                                                }
                                                TokenType::Symbol(sym) if sym == "," => {
                                                    if rev_paren_depth == 0 {
                                                        break; // We hit the comma before our parameter
                                                    } else {
                                                        inner_port.insert_str(0, sym);
                                                    }
                                                }
                                                TokenType::Identifier(i)
                                                | TokenType::Number(i)
                                                | TokenType::StringLiteral(i)
                                                | TokenType::Symbol(i) => {
                                                    inner_port.insert_str(0, i);
                                                }
                                                _ => {}
                                            }
                                        }
                                        inner_port = inner_port.trim().to_string();

                                        let mut outer_map = String::new();
                                        let mut search_idx = local_idx + 1;
                                        let mut paren_depth = 0;

                                        while search_idx < tokens.len() {
                                            match &tokens[search_idx].token_type {
                                                TokenType::Symbol(sym) if sym == "(" => {
                                                    paren_depth += 1;
                                                    outer_map.push_str(sym);
                                                }
                                                TokenType::Symbol(sym) if sym == ")" => {
                                                    if paren_depth == 0 {
                                                        break; // end of map or end of instantiation block
                                                    } else {
                                                        paren_depth -= 1;
                                                        outer_map.push_str(sym);
                                                    }
                                                }
                                                TokenType::Symbol(sym)
                                                    if sym == "," && paren_depth == 0 =>
                                                {
                                                    break;
                                                }
                                                TokenType::Identifier(i)
                                                | TokenType::Number(i)
                                                | TokenType::StringLiteral(i)
                                                | TokenType::Symbol(i) => {
                                                    if !outer_map.is_empty()
                                                        && !matches!(
                                                            &tokens[search_idx].token_type,
                                                            TokenType::Symbol(_)
                                                        )
                                                    {
                                                        outer_map.push(' ');
                                                    }
                                                    outer_map.push_str(i);
                                                }
                                                _ => {}
                                            }
                                            search_idx += 1;
                                        }

                                        if !inner_port.is_empty() {
                                            port_map
                                                .insert(inner_port, outer_map.trim().to_string());
                                        }

                                        local_idx = search_idx; // Jump to `,` or `)`
                                        continue;
                                    }
                                    _ => {}
                                }
                                local_idx += 1;
                            }

                            idx = local_idx;

                            let mut multiplier: usize = generate_stack.iter().product();
                            if multiplier == 0 {
                                multiplier = 1;
                            }

                            for _ in 0..multiplier {
                                self.entities[arch_idx].instantiations.push(Instantiation {
                                    entity_name: target_name.clone(),
                                    ports: Vec::new(),
                                    port_map: port_map.clone(),
                                    file_name: file_name.to_string(),
                                    line: inst_line,
                                });
                            }
                        } else {
                            // Rollback idx entirely since this wasn't an instantiation!
                            idx = start_idx;
                        }
                    }
                }

                _ => {}
            }
            idx += 1;
        }
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
