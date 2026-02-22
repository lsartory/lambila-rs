use serde::Serialize;

use crate::vhdl_token::{TokenType, VhdlToken};
use crate::vhdl_type::{VhdlType, parse_type};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum PortDirection {
    In,
    Out,
    InOut,
    Buffer,
    Linkage,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct VhdlPort {
    pub name: String,
    pub direction: PortDirection,
    pub vhdl_type: VhdlType,
}

/// Parse a `port ( ... )` block from the token stream.
/// `tokens[idx]` should point to the `port` keyword.
/// Returns `(Vec<VhdlPort>, new_idx)` where new_idx is past the closing `)` and `;`.
pub fn parse_ports(tokens: &[VhdlToken], start_idx: usize) -> (Vec<VhdlPort>, usize) {
    let mut idx = start_idx;
    let mut ports = Vec::new();

    // Expect 'port'
    if idx >= tokens.len() {
        return (ports, idx);
    }
    if let TokenType::Identifier(id) = &tokens[idx].token_type {
        if !id.eq_ignore_ascii_case("port") {
            return (ports, idx);
        }
    } else {
        return (ports, idx);
    }
    idx += 1;

    // Expect '('
    if idx >= tokens.len() {
        return (ports, idx);
    }
    if let TokenType::Symbol(s) = &tokens[idx].token_type {
        if s != "(" {
            return (ports, idx);
        }
    } else {
        return (ports, idx);
    }
    idx += 1;

    let mut paren_depth: i32 = 1;

    while idx < tokens.len() && paren_depth > 0 {
        // Collect port names separated by commas until ':'
        let mut names = Vec::new();
        while idx < tokens.len() {
            match &tokens[idx].token_type {
                TokenType::Identifier(name) => {
                    names.push(name.clone());
                    idx += 1;
                    if idx < tokens.len()
                        && let TokenType::Symbol(s) = &tokens[idx].token_type
                    {
                        if s == "," {
                            idx += 1;
                            continue;
                        } else if s == ":" {
                            idx += 1;
                            break;
                        }
                    }
                }
                TokenType::Symbol(s) if s == ")" => {
                    paren_depth -= 1;
                    idx += 1;
                    break;
                }
                _ => {
                    idx += 1;
                }
            }
        }

        if names.is_empty() || paren_depth == 0 {
            continue;
        }

        // Parse direction
        let mut direction = None;
        if idx < tokens.len()
            && let TokenType::Identifier(m) = &tokens[idx].token_type
        {
            direction = match m.to_lowercase().as_str() {
                "in" => Some(PortDirection::In),
                "out" => Some(PortDirection::Out),
                "inout" => Some(PortDirection::InOut),
                "buffer" => Some(PortDirection::Buffer),
                "linkage" => Some(PortDirection::Linkage),
                _ => None,
            };
            if direction.is_some() {
                idx += 1;
            }
        }

        // Parse type (stops at ';', ':=', or ')')
        let (vhdl_type, new_idx) = parse_type(tokens, idx);
        idx = new_idx;

        // Skip default value if present
        if idx < tokens.len()
            && let TokenType::Symbol(s) = &tokens[idx].token_type
            && s == ":="
        {
            let mut init_paren_depth = 0;
            while idx < tokens.len() {
                match &tokens[idx].token_type {
                    TokenType::Symbol(s) if s == "(" => {
                        init_paren_depth += 1;
                    }
                    TokenType::Symbol(s) if s == ")" => {
                        if init_paren_depth > 0 {
                            init_paren_depth -= 1;
                        } else {
                            break;
                        }
                    }
                    TokenType::Symbol(s) if s == ";" => {
                        break;
                    }
                    _ => {}
                }
                idx += 1;
            }
        }

        // Consume ';' or ')'
        if idx < tokens.len()
            && let TokenType::Symbol(s) = &tokens[idx].token_type
        {
            if s == ";" {
                idx += 1;
            } else if s == ")" {
                paren_depth -= 1;
                idx += 1;
            }
        }

        // Create ports
        if let Some(dir) = direction {
            for name in names {
                ports.push(VhdlPort {
                    name,
                    direction: dir.clone(),
                    vhdl_type: vhdl_type.clone(),
                });
            }
        }
    }

    // Skip trailing ';' if present
    if idx < tokens.len()
        && let TokenType::Symbol(s) = &tokens[idx].token_type
        && s == ";"
    {
        idx += 1;
    }

    (ports, idx)
}
