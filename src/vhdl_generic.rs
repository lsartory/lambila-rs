use serde::Serialize;

use crate::vhdl_token::{TokenType, VhdlToken};
use crate::vhdl_type::{VhdlType, parse_type};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct VhdlGeneric {
    pub name: String,
    pub vhdl_type: VhdlType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_value: Option<String>,
}

/// Parse a `generic ( ... )` block from the token stream.
/// `tokens[idx]` should point to the `generic` keyword.
/// Returns `(Vec<VhdlGeneric>, new_idx)` where new_idx is past the closing `)` and `;`.
pub fn parse_generics(tokens: &[VhdlToken], start_idx: usize) -> (Vec<VhdlGeneric>, usize) {
    let mut idx = start_idx;
    let mut generics = Vec::new();

    // Expect 'generic'
    if idx >= tokens.len() {
        return (generics, idx);
    }
    if let TokenType::Identifier(id) = &tokens[idx].token_type {
        if !id.eq_ignore_ascii_case("generic") {
            return (generics, idx);
        }
    } else {
        return (generics, idx);
    }
    idx += 1;

    // Expect '('
    if idx >= tokens.len() {
        return (generics, idx);
    }
    if let TokenType::Symbol(s) = &tokens[idx].token_type {
        if s != "(" {
            return (generics, idx);
        }
    } else {
        return (generics, idx);
    }
    idx += 1;

    let mut paren_depth = 1;

    while idx < tokens.len() && paren_depth > 0 {
        // Collect generic names separated by commas until ':'
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

        // Parse type
        let (vhdl_type, new_idx) = parse_type(tokens, idx);
        idx = new_idx;

        // Check for default value ':='
        let mut default_value = None;
        if idx < tokens.len()
            && let TokenType::Symbol(s) = &tokens[idx].token_type
        {
            if s == ":=" {
                idx += 1;
                let mut val = String::new();
                let mut val_paren_depth = 0;
                while idx < tokens.len() {
                    match &tokens[idx].token_type {
                        TokenType::Symbol(s) if s == "(" => {
                            val_paren_depth += 1;
                            val.push_str(s);
                        }
                        TokenType::Symbol(s) if s == ")" => {
                            if val_paren_depth > 0 {
                                val_paren_depth -= 1;
                                val.push_str(s);
                            } else {
                                // End of generic block
                                paren_depth -= 1;
                                idx += 1;
                                break;
                            }
                        }
                        TokenType::Symbol(s) if s == ";" => {
                            idx += 1;
                            break;
                        }
                        TokenType::Identifier(i)
                        | TokenType::Number(i)
                        | TokenType::StringLiteral(i)
                        | TokenType::Symbol(i) => {
                            if !val.is_empty() {
                                val.push(' ');
                            }
                            val.push_str(i);
                        }
                        TokenType::CharLiteral(c) => {
                            if !val.is_empty() {
                                val.push(' ');
                            }
                            val.push('\'');
                            val.push(*c);
                            val.push('\'');
                        }
                    }
                    idx += 1;
                }
                if !val.is_empty() {
                    default_value = Some(val.trim().to_string());
                }
            } else if s == ";" {
                idx += 1;
            } else if s == ")" {
                paren_depth -= 1;
                idx += 1;
            }
        }

        for name in names {
            generics.push(VhdlGeneric {
                name,
                vhdl_type: vhdl_type.clone(),
                default_value: default_value.clone(),
                actual_value: None,
            });
        }
    }

    // Skip trailing ';' if present
    if idx < tokens.len()
        && let TokenType::Symbol(s) = &tokens[idx].token_type
        && s == ";"
    {
        idx += 1;
    }

    (generics, idx)
}
