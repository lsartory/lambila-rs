use serde::Serialize;

use crate::vhdl_token::{TokenType, VhdlToken};
use crate::vhdl_type::{VhdlType, parse_type};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct VhdlConstant {
    pub name: String,
    pub vhdl_type: VhdlType,
    pub value: String,
}

/// Parse a `constant` declaration from the token stream.
/// `tokens[idx]` should point to the `constant` keyword.
/// Returns `(Vec<VhdlConstant>, new_idx)` where new_idx is past the trailing ';'.
pub fn parse_constants(tokens: &[VhdlToken], start_idx: usize) -> (Vec<VhdlConstant>, usize) {
    let mut idx = start_idx;
    let mut constants = Vec::new();

    // Skip 'constant' keyword
    idx += 1;

    // Collect names until ':'
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
            _ => {
                idx += 1;
            }
        }
    }

    if names.is_empty() {
        // Skip to ';'
        while idx < tokens.len() {
            if let TokenType::Symbol(s) = &tokens[idx].token_type
                && s == ";"
            {
                idx += 1;
                break;
            }
            idx += 1;
        }
        return (constants, idx);
    }

    // Parse type
    let (vhdl_type, new_idx) = parse_type(tokens, idx);
    idx = new_idx;

    // Parse default value after ':='
    let mut value = String::new();
    if idx < tokens.len()
        && let TokenType::Symbol(s) = &tokens[idx].token_type
        && s == ":="
    {
        idx += 1;
        let mut paren_depth = 0;
        while idx < tokens.len() {
            match &tokens[idx].token_type {
                TokenType::Symbol(s) if s == "(" => {
                    paren_depth += 1;
                    value.push_str(s);
                }
                TokenType::Symbol(s) if s == ")" => {
                    if paren_depth == 0 {
                        break;
                    }
                    paren_depth -= 1;
                    value.push_str(s);
                }
                TokenType::Symbol(s) if s == ";" && paren_depth == 0 => {
                    break;
                }
                TokenType::Identifier(i)
                | TokenType::Number(i)
                | TokenType::StringLiteral(i)
                | TokenType::Symbol(i) => {
                    if !value.is_empty() && !matches!(&tokens[idx].token_type, TokenType::Symbol(_))
                    {
                        value.push(' ');
                    }
                    value.push_str(i);
                }
                TokenType::CharLiteral(c) => {
                    if !value.is_empty() {
                        value.push(' ');
                    }
                    value.push('\'');
                    value.push(*c);
                    value.push('\'');
                }
            }
            idx += 1;
        }
    }

    // Consume trailing ';'
    if idx < tokens.len()
        && let TokenType::Symbol(s) = &tokens[idx].token_type
        && s == ";"
    {
        idx += 1;
    }

    let value = value.trim().to_string();
    for name in names {
        constants.push(VhdlConstant {
            name,
            vhdl_type: vhdl_type.clone(),
            value: value.clone(),
        });
    }

    (constants, idx)
}
