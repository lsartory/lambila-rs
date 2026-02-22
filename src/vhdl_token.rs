//! VHDL Token types produced by the lexer.

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Identifier(String),    // Words, keywords (e.g., "entity", "std_logic")
    Number(String),        // Numbers (e.g., "105", "0", "16#FF#")
    StringLiteral(String), // String literals
    CharLiteral(char),     // Character literals (e.g. '0')
    Symbol(String),        // Operators/symbols: "(", ")", ":", ";", ",", "=>", etc.
}

#[derive(Debug, Clone)]
pub struct VhdlToken {
    pub token_type: TokenType,
    pub line: usize,
}

impl VhdlToken {
    pub fn tokenize(input: &str) -> Vec<VhdlToken> {
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
                        while let Some(&nc) = chars.peek() {
                            if nc == '\n' {
                                break;
                            }
                            chars.next();
                        }
                    } else {
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol("-".to_string()),
                            line,
                        });
                    }
                }
                '=' => {
                    chars.next();
                    if chars.peek() == Some(&'>') {
                        chars.next();
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol("=>".to_string()),
                            line,
                        });
                    } else {
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol("=".to_string()),
                            line,
                        });
                    }
                }
                ':' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol(":=".to_string()),
                            line,
                        });
                    } else {
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol(":".to_string()),
                            line,
                        });
                    }
                }
                '/' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol("/=".to_string()),
                            line,
                        });
                    } else {
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol("/".to_string()),
                            line,
                        });
                    }
                }
                '<' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol("<=".to_string()),
                            line,
                        });
                    } else {
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol("<".to_string()),
                            line,
                        });
                    }
                }
                '>' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol(">=".to_string()),
                            line,
                        });
                    } else {
                        tokens.push(VhdlToken {
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
                            chars.next();
                            break;
                        } else if nc == '\n' {
                            line += 1;
                        }
                        s.push(nc);
                        chars.next();
                    }
                    tokens.push(VhdlToken {
                        token_type: TokenType::StringLiteral(s),
                        line,
                    });
                }
                '\'' => {
                    chars.next();
                    if let Some(&nc) = chars.peek() {
                        if nc.is_alphabetic() {
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
                            tokens.push(VhdlToken {
                                token_type: TokenType::Identifier(ident),
                                line,
                            });
                        } else {
                            let val = nc;
                            chars.next();
                            if chars.peek() == Some(&'\'') {
                                chars.next();
                                tokens.push(VhdlToken {
                                    token_type: TokenType::CharLiteral(val),
                                    line,
                                });
                            } else {
                                tokens.push(VhdlToken {
                                    token_type: TokenType::Symbol("'".to_string()),
                                    line,
                                });
                            }
                        }
                    } else {
                        tokens.push(VhdlToken {
                            token_type: TokenType::Symbol("'".to_string()),
                            line,
                        });
                    }
                }
                '(' | ')' | ';' | ',' | '+' | '*' | '&' | '|' | '.' => {
                    tokens.push(VhdlToken {
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
                    tokens.push(VhdlToken {
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
                            num.push(nc);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(VhdlToken {
                        token_type: TokenType::Number(num),
                        line,
                    });
                }
                _ => {
                    tokens.push(VhdlToken {
                        token_type: TokenType::Symbol(c.to_string()),
                        line,
                    });
                    chars.next();
                }
            }
        }
        tokens
    }
}
