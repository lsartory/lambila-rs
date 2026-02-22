use serde::Serialize;

use crate::vhdl_token::{TokenType, VhdlToken};
use crate::vhdl_type::{VhdlType, parse_type};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct VhdlSignal {
    pub name: String,
    pub vhdl_type: VhdlType,
}

/// Parse a `signal` declaration from the token stream.
/// `tokens[idx]` should point to the `signal` keyword.
/// Returns `(Vec<VhdlSignal>, new_idx)` where new_idx is past the trailing ';'.
pub fn parse_signals(tokens: &[VhdlToken], start_idx: usize) -> (Vec<VhdlSignal>, usize) {
    let mut idx = start_idx;
    let mut signals = Vec::new();

    // Skip 'signal' keyword
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
        return (signals, idx);
    }

    // Parse type
    let (vhdl_type, new_idx) = parse_type(tokens, idx);
    idx = new_idx;

    // Skip default value and consume until ';'
    while idx < tokens.len() {
        if let TokenType::Symbol(s) = &tokens[idx].token_type
            && s == ";"
        {
            idx += 1;
            break;
        }
        idx += 1;
    }

    for name in names {
        signals.push(VhdlSignal {
            name,
            vhdl_type: vhdl_type.clone(),
        });
    }

    (signals, idx)
}
