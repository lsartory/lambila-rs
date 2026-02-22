use serde::Serialize;

use crate::vhdl_range::{VhdlRange, parse_range_tokens};
use crate::vhdl_token::{TokenType, VhdlToken};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct VhdlType {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<VhdlRange>,
}

/// Parse a VHDL type from a token stream.
/// Starts at `tokens[idx]` which should be the first token of the type name.
/// Stops at `;`, `:=`, or `)` (without consuming those).
/// Returns `(VhdlType, new_idx)`.
pub fn parse_type(tokens: &[VhdlToken], start_idx: usize) -> (VhdlType, usize) {
    let mut idx = start_idx;
    let mut name = String::new();
    let mut range: Option<VhdlRange> = None;

    while idx < tokens.len() {
        match &tokens[idx].token_type {
            TokenType::Symbol(s) if s == ";" || s == ":=" || s == ")" => {
                break;
            }
            TokenType::Symbol(s) if s == "(" => {
                idx += 1; // skip '('
                let (fragment, parsed_range, new_idx) = parse_range_tokens(tokens, idx);
                if parsed_range.is_some() {
                    range = parsed_range;
                } else {
                    name.push_str(&fragment);
                }
                idx = new_idx;
                continue;
            }
            TokenType::Identifier(i) | TokenType::Number(i) | TokenType::Symbol(i) => {
                name.push_str(i);
                name.push(' ');
            }
            _ => {}
        }
        idx += 1;
    }

    (
        VhdlType {
            name: name.trim().to_string(),
            range,
        },
        idx,
    )
}
