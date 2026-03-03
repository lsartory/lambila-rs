//! Parser infrastructure for VHDL token streams.

use crate::{KeywordKind, Span, Token, TokenKind};

/// A parse error with optional source location.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Human-readable error message.
    pub message: String,
    /// Location of the error in the source, if available.
    pub span: Option<Span>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(span) = &self.span {
            write!(f, "{}:{}: {}", span.line, span.col, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}

/// A cursor-based parser over a slice of VHDL tokens.
pub struct Parser<'a> {
    tokens: &'a [Token],
    cursor: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser over the given token slice.
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, cursor: 0 }
    }

    /// Peek at the current token without consuming it.
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    /// Peek at the n-th token ahead (0 = current).
    pub fn peek_nth(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.cursor + n)
    }

    /// Get the `TokenKind` of the current token, or `None` at EOF.
    pub fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind)
    }

    /// Returns `true` if the current token matches the given kind.
    pub fn at(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(kind)
    }

    /// Returns `true` if the current token is the given keyword.
    pub fn at_keyword(&self, kw: KeywordKind) -> bool {
        self.at(TokenKind::Keyword(kw))
    }

    /// Consume the current token and advance the cursor.
    pub fn consume(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.cursor);
        if token.is_some() {
            self.cursor += 1;
        }
        token
    }

    /// Consume the current token if it matches the given kind.
    /// Returns the token if consumed, `None` otherwise.
    pub fn consume_if(&mut self, kind: TokenKind) -> Option<&Token> {
        if self.at(kind) { self.consume() } else { None }
    }

    /// Consume the current token if it is the given keyword.
    /// Returns the token if consumed, `None` otherwise.
    pub fn consume_if_keyword(&mut self, kw: KeywordKind) -> Option<&Token> {
        self.consume_if(TokenKind::Keyword(kw))
    }

    /// Expect the current token to be of the given kind, consume it,
    /// and return a reference to it. Returns a `ParseError` on mismatch.
    pub fn expect(&mut self, kind: TokenKind) -> Result<&Token, ParseError> {
        match self.peek() {
            Some(token) if token.kind == kind => {
                self.cursor += 1;
                Ok(&self.tokens[self.cursor - 1])
            }
            Some(token) => Err(ParseError {
                message: format!("expected {:?}, found {:?}", kind, token.kind),
                span: Some(token.span),
            }),
            None => Err(ParseError {
                message: format!("expected {:?}, found end of input", kind),
                span: None,
            }),
        }
    }

    /// Expect the current token to be the given keyword, consume it,
    /// and return a reference to it.
    pub fn expect_keyword(&mut self, kw: KeywordKind) -> Result<&Token, ParseError> {
        self.expect(TokenKind::Keyword(kw))
    }

    /// Expect an identifier token, consume it, and return its text.
    pub fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(token)
                if token.kind == TokenKind::Identifier
                    || token.kind == TokenKind::ExtendedIdentifier =>
            {
                let text = token.text.clone();
                self.cursor += 1;
                Ok(text)
            }
            Some(token) => Err(ParseError {
                message: format!("expected identifier, found {:?}", token.kind),
                span: Some(token.span),
            }),
            None => Err(ParseError {
                message: "expected identifier, found end of input".to_string(),
                span: None,
            }),
        }
    }

    /// Save the current cursor position for backtracking.
    pub fn save(&self) -> usize {
        self.cursor
    }

    /// Restore the cursor to a previously saved position.
    pub fn restore(&mut self, pos: usize) {
        self.cursor = pos;
    }

    /// Create a `ParseError` at the current token position.
    pub fn error(&self, msg: impl Into<String>) -> ParseError {
        ParseError {
            message: msg.into(),
            span: self.peek().map(|t| t.span),
        }
    }

    /// Returns `true` if the parser has reached the end of input.
    pub fn eof(&self) -> bool {
        self.cursor >= self.tokens.len()
            || self
                .tokens
                .get(self.cursor)
                .is_none_or(|t| t.kind == TokenKind::Eof)
    }
}

#[cfg(test)]
mod tests {
    use crate::{VhdlVersion, lex, parse_str};

    fn roundtrip(source: &str) -> String {
        let design_file = parse_str(source, VhdlVersion::Vhdl2008).unwrap_or_else(|e| {
            panic!("parse failed: {e}\nsource:\n{source}");
        });
        format!("{design_file}")
    }

    fn roundtrip_ok(source: &str) {
        let formatted = roundtrip(source);
        // Re-parse the formatted output to verify it's valid
        parse_str(&formatted, VhdlVersion::Vhdl2008).unwrap_or_else(|e| {
            panic!("re-parse of formatted output failed: {e}\nformatted:\n{formatted}");
        });
    }

    #[test]
    fn test_empty_entity() {
        roundtrip_ok("entity e is end entity e;");
    }

    #[test]
    fn test_entity_with_ports() {
        roundtrip_ok(
            "entity my_ent is\n\
             port (\n\
               clk : in std_logic;\n\
               data : out std_logic_vector(7 downto 0)\n\
             );\n\
             end entity my_ent;",
        );
    }

    #[test]
    fn test_entity_with_generics() {
        roundtrip_ok(
            "entity my_ent is\n\
             generic (\n\
               WIDTH : integer := 8\n\
             );\n\
             port (\n\
               data : out std_logic_vector(WIDTH - 1 downto 0)\n\
             );\n\
             end entity my_ent;",
        );
    }

    #[test]
    fn test_architecture_empty() {
        roundtrip_ok(
            "entity e is end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_signal_assignment() {
        roundtrip_ok(
            "entity e is\n\
             port (a, b : in std_logic; y : out std_logic);\n\
             end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               y <= a and b;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_conditional_signal_assignment() {
        roundtrip_ok(
            "entity e is\n\
             port (sel : in std_logic; a, b : in std_logic; y : out std_logic);\n\
             end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               y <= a when sel = '1' else b;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_process_with_if() {
        roundtrip_ok(
            "entity e is\n\
             port (clk : in std_logic; q : out std_logic);\n\
             end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               process (clk)\n\
               begin\n\
                 if clk = '1' then\n\
                   q <= '1';\n\
                 end if;\n\
               end process;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_process_with_case() {
        roundtrip_ok(
            "entity e is\n\
             port (sel : in std_logic_vector(1 downto 0); y : out std_logic);\n\
             end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               process (sel)\n\
               begin\n\
                 case sel is\n\
                   when \"00\" => y <= '0';\n\
                   when \"01\" => y <= '1';\n\
                   when others => y <= '0';\n\
                 end case;\n\
               end process;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_library_and_use() {
        roundtrip_ok(
            "library ieee;\n\
             use ieee.std_logic_1164.all;\n\
             entity e is end entity e;",
        );
    }

    #[test]
    fn test_package_declaration() {
        roundtrip_ok(
            "package my_pkg is\n\
             constant C : integer := 42;\n\
             end package my_pkg;",
        );
    }

    #[test]
    fn test_package_body() {
        roundtrip_ok(
            "package my_pkg is\n\
             function f(x : integer) return integer;\n\
             end package my_pkg;\n\
             package body my_pkg is\n\
             function f(x : integer) return integer is\n\
             begin\n\
               return x + 1;\n\
             end function f;\n\
             end package body my_pkg;",
        );
    }

    #[test]
    fn test_component_declaration() {
        roundtrip_ok(
            "entity e is end entity e;\n\
             architecture rtl of e is\n\
             component comp is\n\
               port (x : in std_logic);\n\
             end component comp;\n\
             begin\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_type_declarations() {
        roundtrip_ok(
            "package p is\n\
             type color_t is (RED, GREEN, BLUE);\n\
             type byte is range 0 to 255;\n\
             subtype nibble is byte range 0 to 15;\n\
             end package p;",
        );
    }

    #[test]
    fn test_record_type() {
        roundtrip_ok(
            "package p is\n\
             type point is record\n\
               x : integer;\n\
               y : integer;\n\
             end record;\n\
             end package p;",
        );
    }

    #[test]
    fn test_array_type() {
        roundtrip_ok(
            "package p is\n\
             type mem_t is array (0 to 255) of std_logic_vector(7 downto 0);\n\
             end package p;",
        );
    }

    #[test]
    fn test_for_loop() {
        roundtrip_ok(
            "entity e is end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               process\n\
               begin\n\
                 for i in 0 to 7 loop\n\
                   null;\n\
                 end loop;\n\
               end process;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_while_loop() {
        roundtrip_ok(
            "entity e is end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               process\n\
               begin\n\
                 while true loop\n\
                   wait;\n\
                 end loop;\n\
               end process;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_generate_statement() {
        roundtrip_ok(
            "entity e is end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               gen : for i in 0 to 3 generate\n\
               end generate gen;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_expression_precedence() {
        // Test that complex expressions parse and roundtrip
        roundtrip_ok(
            "entity e is\n\
             port (a, b, c : in integer; y : out integer);\n\
             end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               y <= a + b * c;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_aggregate() {
        roundtrip_ok(
            "entity e is\n\
             port (y : out std_logic_vector(7 downto 0));\n\
             end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               y <= (others => '0');\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_variable_assignment() {
        roundtrip_ok(
            "entity e is end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               process\n\
               variable v : integer;\n\
               begin\n\
                 v := 42;\n\
               end process;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_multiple_design_units() {
        roundtrip_ok(
            "library ieee;\n\
             use ieee.std_logic_1164.all;\n\
             entity e1 is end entity e1;\n\
             architecture rtl of e1 is begin end architecture rtl;\n\
             entity e2 is end entity e2;\n\
             architecture rtl of e2 is begin end architecture rtl;",
        );
    }

    #[test]
    fn test_assert_statement() {
        roundtrip_ok(
            "entity e is end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               process\n\
               begin\n\
                 assert false report \"test\" severity error;\n\
               end process;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_wait_statement() {
        roundtrip_ok(
            "entity e is end entity e;\n\
             architecture rtl of e is\n\
             begin\n\
               process\n\
               begin\n\
                 wait for 10 ns;\n\
               end process;\n\
             end architecture rtl;",
        );
    }

    #[test]
    fn test_lex_command() {
        // Ensure the lex function still works (no regression)
        let result = lex("entity e is end;", VhdlVersion::Vhdl1993);
        assert!(result.errors.is_empty());
        assert!(result.tokens.len() > 1);
    }
}
