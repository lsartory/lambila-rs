use serde::Serialize;

use crate::vhdl_token::{TokenType, VhdlToken};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Direction {
    To,
    Downto,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct VhdlRange {
    pub left: String,
    pub direction: Direction,
    pub right: String,
}

/// Parse a range from tokens inside parentheses.
/// `tokens[idx]` should be the first token after the opening '('.
/// Returns `(data_type_fragment, Option<VhdlRange>, new_idx)`.
/// `new_idx` points to the token after the closing ')'.
pub fn parse_range_tokens(
    tokens: &[VhdlToken],
    start_idx: usize,
) -> (String, Option<VhdlRange>, usize) {
    let mut idx = start_idx;
    let mut inner_depth: i32 = 1;
    let mut range_tokens = Vec::new();
    let mut data_type_fragment = String::from("(");

    while idx < tokens.len() && inner_depth > 0 {
        match &tokens[idx].token_type {
            TokenType::Symbol(s) if s == "(" => {
                inner_depth += 1;
            }
            TokenType::Symbol(s) if s == ")" => {
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
                    data_type_fragment.push_str(i);
                    if !matches!(&tokens[idx].token_type, TokenType::Symbol(_)) {
                        data_type_fragment.push(' ');
                    }
                }
                _ => {}
            }
        } else {
            data_type_fragment = data_type_fragment.trim_end().to_string();
            data_type_fragment.push(')');
        }
        idx += 1;
    }

    // Try to extract a range (left direction right)
    let mut r_left = String::new();
    let mut r_right = String::new();
    let mut r_dir = None;
    let mut hit_dir = false;

    for rt in &range_tokens {
        match &rt.token_type {
            TokenType::Identifier(dir) if dir.eq_ignore_ascii_case("downto") => {
                r_dir = Some(Direction::Downto);
                hit_dir = true;
            }
            TokenType::Identifier(dir) if dir.eq_ignore_ascii_case("to") => {
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
        let range = VhdlRange {
            left: r_left.trim().to_string(),
            direction: d,
            right: r_right.trim().to_string(),
        };
        (data_type_fragment, Some(range), idx)
    } else {
        (data_type_fragment, None, idx)
    }
}

impl VhdlRange {
    /// Simple range size parser for for-generate loops.
    /// Parses expressions like "0 to 7" or "15 downto 0" and returns the count.
    pub fn parse_range_size_simple(range_expr: &str) -> usize {
        let expr = range_expr.to_lowercase();

        let mut parts = Vec::new();
        if let Some(idx) = expr.find("downto") {
            parts.push(&expr[..idx]);
            parts.push(&expr[idx + 6..]);
        } else if let Some(idx) = expr.find("to") {
            parts.push(&expr[..idx]);
            parts.push(&expr[idx + 2..]);
        }

        if parts.len() >= 2 {
            let left = Self::extract_last_number(parts[0]);
            let right = Self::extract_first_number(parts[1]);
            if let (Some(l), Some(r)) = (left, right) {
                return (l - r).unsigned_abs() as usize + 1;
            }
        }

        1
    }

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
}
