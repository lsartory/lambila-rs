use crate::token::{KeywordKind, TokenKind};
use crate::version::VhdlVersion;

/// Look up whether `word` (case-insensitive) is a reserved keyword in the
/// given VHDL version. Returns `Some(TokenKind::Keyword(..))` if it is,
/// `None` otherwise.
pub fn lookup_keyword(word: &str, version: VhdlVersion) -> Option<TokenKind> {
    // Convert to lowercase for case-insensitive matching.
    let lower: String = word.to_ascii_lowercase();

    // First, look up the keyword and its minimum required version.
    let (kind, min_version) = match lower.as_str() {
        // ── VHDL-1987 keywords (81) ───────────────────────────────────────
        "abs" => (KeywordKind::Abs, VhdlVersion::Vhdl1987),
        "access" => (KeywordKind::Access, VhdlVersion::Vhdl1987),
        "after" => (KeywordKind::After, VhdlVersion::Vhdl1987),
        "alias" => (KeywordKind::Alias, VhdlVersion::Vhdl1987),
        "all" => (KeywordKind::All, VhdlVersion::Vhdl1987),
        "and" => (KeywordKind::And, VhdlVersion::Vhdl1987),
        "architecture" => (KeywordKind::Architecture, VhdlVersion::Vhdl1987),
        "array" => (KeywordKind::Array, VhdlVersion::Vhdl1987),
        "assert" => (KeywordKind::Assert, VhdlVersion::Vhdl1987),
        "attribute" => (KeywordKind::Attribute, VhdlVersion::Vhdl1987),
        "begin" => (KeywordKind::Begin, VhdlVersion::Vhdl1987),
        "block" => (KeywordKind::Block, VhdlVersion::Vhdl1987),
        "body" => (KeywordKind::Body, VhdlVersion::Vhdl1987),
        "buffer" => (KeywordKind::Buffer, VhdlVersion::Vhdl1987),
        "bus" => (KeywordKind::Bus, VhdlVersion::Vhdl1987),
        "case" => (KeywordKind::Case, VhdlVersion::Vhdl1987),
        "component" => (KeywordKind::Component, VhdlVersion::Vhdl1987),
        "configuration" => (KeywordKind::Configuration, VhdlVersion::Vhdl1987),
        "constant" => (KeywordKind::Constant, VhdlVersion::Vhdl1987),
        "disconnect" => (KeywordKind::Disconnect, VhdlVersion::Vhdl1987),
        "downto" => (KeywordKind::Downto, VhdlVersion::Vhdl1987),
        "else" => (KeywordKind::Else, VhdlVersion::Vhdl1987),
        "elsif" => (KeywordKind::Elsif, VhdlVersion::Vhdl1987),
        "end" => (KeywordKind::End, VhdlVersion::Vhdl1987),
        "entity" => (KeywordKind::Entity, VhdlVersion::Vhdl1987),
        "exit" => (KeywordKind::Exit, VhdlVersion::Vhdl1987),
        "file" => (KeywordKind::File, VhdlVersion::Vhdl1987),
        "for" => (KeywordKind::For, VhdlVersion::Vhdl1987),
        "function" => (KeywordKind::Function, VhdlVersion::Vhdl1987),
        "generate" => (KeywordKind::Generate, VhdlVersion::Vhdl1987),
        "generic" => (KeywordKind::Generic, VhdlVersion::Vhdl1987),
        "guarded" => (KeywordKind::Guarded, VhdlVersion::Vhdl1987),
        "if" => (KeywordKind::If, VhdlVersion::Vhdl1987),
        "in" => (KeywordKind::In, VhdlVersion::Vhdl1987),
        "inout" => (KeywordKind::Inout, VhdlVersion::Vhdl1987),
        "is" => (KeywordKind::Is, VhdlVersion::Vhdl1987),
        "label" => (KeywordKind::Label, VhdlVersion::Vhdl1987),
        "library" => (KeywordKind::Library, VhdlVersion::Vhdl1987),
        "linkage" => (KeywordKind::Linkage, VhdlVersion::Vhdl1987),
        "loop" => (KeywordKind::Loop, VhdlVersion::Vhdl1987),
        "map" => (KeywordKind::Map, VhdlVersion::Vhdl1987),
        "mod" => (KeywordKind::Mod, VhdlVersion::Vhdl1987),
        "nand" => (KeywordKind::Nand, VhdlVersion::Vhdl1987),
        "new" => (KeywordKind::New, VhdlVersion::Vhdl1987),
        "next" => (KeywordKind::Next, VhdlVersion::Vhdl1987),
        "nor" => (KeywordKind::Nor, VhdlVersion::Vhdl1987),
        "not" => (KeywordKind::Not, VhdlVersion::Vhdl1987),
        "null" => (KeywordKind::Null, VhdlVersion::Vhdl1987),
        "of" => (KeywordKind::Of, VhdlVersion::Vhdl1987),
        "on" => (KeywordKind::On, VhdlVersion::Vhdl1987),
        "open" => (KeywordKind::Open, VhdlVersion::Vhdl1987),
        "or" => (KeywordKind::Or, VhdlVersion::Vhdl1987),
        "others" => (KeywordKind::Others, VhdlVersion::Vhdl1987),
        "out" => (KeywordKind::Out, VhdlVersion::Vhdl1987),
        "package" => (KeywordKind::Package, VhdlVersion::Vhdl1987),
        "port" => (KeywordKind::Port, VhdlVersion::Vhdl1987),
        "procedure" => (KeywordKind::Procedure, VhdlVersion::Vhdl1987),
        "process" => (KeywordKind::Process, VhdlVersion::Vhdl1987),
        "range" => (KeywordKind::Range, VhdlVersion::Vhdl1987),
        "record" => (KeywordKind::Record, VhdlVersion::Vhdl1987),
        "register" => (KeywordKind::Register, VhdlVersion::Vhdl1987),
        "rem" => (KeywordKind::Rem, VhdlVersion::Vhdl1987),
        "report" => (KeywordKind::Report, VhdlVersion::Vhdl1987),
        "return" => (KeywordKind::Return, VhdlVersion::Vhdl1987),
        "select" => (KeywordKind::Select, VhdlVersion::Vhdl1987),
        "severity" => (KeywordKind::Severity, VhdlVersion::Vhdl1987),
        "signal" => (KeywordKind::Signal, VhdlVersion::Vhdl1987),
        "subtype" => (KeywordKind::Subtype, VhdlVersion::Vhdl1987),
        "then" => (KeywordKind::Then, VhdlVersion::Vhdl1987),
        "to" => (KeywordKind::To, VhdlVersion::Vhdl1987),
        "transport" => (KeywordKind::Transport, VhdlVersion::Vhdl1987),
        "type" => (KeywordKind::Type, VhdlVersion::Vhdl1987),
        "units" => (KeywordKind::Units, VhdlVersion::Vhdl1987),
        "until" => (KeywordKind::Until, VhdlVersion::Vhdl1987),
        "use" => (KeywordKind::Use, VhdlVersion::Vhdl1987),
        "variable" => (KeywordKind::Variable, VhdlVersion::Vhdl1987),
        "wait" => (KeywordKind::Wait, VhdlVersion::Vhdl1987),
        "when" => (KeywordKind::When, VhdlVersion::Vhdl1987),
        "while" => (KeywordKind::While, VhdlVersion::Vhdl1987),
        "with" => (KeywordKind::With, VhdlVersion::Vhdl1987),
        "xor" => (KeywordKind::Xor, VhdlVersion::Vhdl1987),

        // ── VHDL-1993 additions (+16) ─────────────────────────────────────
        "group" => (KeywordKind::Group, VhdlVersion::Vhdl1993),
        "impure" => (KeywordKind::Impure, VhdlVersion::Vhdl1993),
        "inertial" => (KeywordKind::Inertial, VhdlVersion::Vhdl1993),
        "literal" => (KeywordKind::Literal, VhdlVersion::Vhdl1993),
        "postponed" => (KeywordKind::Postponed, VhdlVersion::Vhdl1993),
        "pure" => (KeywordKind::Pure, VhdlVersion::Vhdl1993),
        "reject" => (KeywordKind::Reject, VhdlVersion::Vhdl1993),
        "rol" => (KeywordKind::Rol, VhdlVersion::Vhdl1993),
        "ror" => (KeywordKind::Ror, VhdlVersion::Vhdl1993),
        "shared" => (KeywordKind::Shared, VhdlVersion::Vhdl1993),
        "sla" => (KeywordKind::Sla, VhdlVersion::Vhdl1993),
        "sll" => (KeywordKind::Sll, VhdlVersion::Vhdl1993),
        "sra" => (KeywordKind::Sra, VhdlVersion::Vhdl1993),
        "srl" => (KeywordKind::Srl, VhdlVersion::Vhdl1993),
        "unaffected" => (KeywordKind::Unaffected, VhdlVersion::Vhdl1993),
        "xnor" => (KeywordKind::Xnor, VhdlVersion::Vhdl1993),

        // ── VHDL-2008 additions (+19) ───────────────────────────────────
        "assume" => (KeywordKind::Assume, VhdlVersion::Vhdl2008),
        "assume_guarantee" => (KeywordKind::AssumeGuarantee, VhdlVersion::Vhdl2008),
        "context" => (KeywordKind::Context, VhdlVersion::Vhdl2008),
        "cover" => (KeywordKind::Cover, VhdlVersion::Vhdl2008),
        "default" => (KeywordKind::Default, VhdlVersion::Vhdl2008),
        "fairness" => (KeywordKind::Fairness, VhdlVersion::Vhdl2008),
        "force" => (KeywordKind::Force, VhdlVersion::Vhdl2008),
        "inherit" => (KeywordKind::Inherit, VhdlVersion::Vhdl2008),
        "parameter" => (KeywordKind::Parameter, VhdlVersion::Vhdl2008),
        "property" => (KeywordKind::Property, VhdlVersion::Vhdl2008),
        "protected" => (KeywordKind::Protected, VhdlVersion::Vhdl2008),
        "release" => (KeywordKind::Release, VhdlVersion::Vhdl2008),
        "restrict" => (KeywordKind::Restrict, VhdlVersion::Vhdl2008),
        "restrict_guarantee" => (KeywordKind::RestrictGuarantee, VhdlVersion::Vhdl2008),
        "sequence" => (KeywordKind::Sequence, VhdlVersion::Vhdl2008),
        "strong" => (KeywordKind::Strong, VhdlVersion::Vhdl2008),
        "vmode" => (KeywordKind::Vmode, VhdlVersion::Vhdl2008),
        "vprop" => (KeywordKind::Vprop, VhdlVersion::Vhdl2008),
        "vunit" => (KeywordKind::Vunit, VhdlVersion::Vhdl2008),

        _ => return None,
    };

    // Only return the keyword if the current version is >= the minimum version.
    if version >= min_version {
        Some(TokenKind::Keyword(kind))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_keyword_lookup() {
        assert_eq!(
            lookup_keyword("entity", VhdlVersion::Vhdl1987),
            Some(TokenKind::Keyword(KeywordKind::Entity))
        );
        assert_eq!(
            lookup_keyword("ENTITY", VhdlVersion::Vhdl1987),
            Some(TokenKind::Keyword(KeywordKind::Entity))
        );
        assert_eq!(
            lookup_keyword("Entity", VhdlVersion::Vhdl1993),
            Some(TokenKind::Keyword(KeywordKind::Entity))
        );
    }

    #[test]
    fn test_version_gating_93() {
        // XNOR is a VHDL-1993 keyword, not in VHDL-1987
        assert_eq!(lookup_keyword("xnor", VhdlVersion::Vhdl1987), None);
        assert_eq!(
            lookup_keyword("xnor", VhdlVersion::Vhdl1993),
            Some(TokenKind::Keyword(KeywordKind::Xnor))
        );
        assert_eq!(
            lookup_keyword("xnor", VhdlVersion::Vhdl2008),
            Some(TokenKind::Keyword(KeywordKind::Xnor))
        );
    }

    #[test]
    fn test_version_gating_2008() {
        // CONTEXT is a VHDL-2008 keyword
        assert_eq!(lookup_keyword("context", VhdlVersion::Vhdl1987), None);
        assert_eq!(lookup_keyword("context", VhdlVersion::Vhdl1993), None);
        assert_eq!(
            lookup_keyword("context", VhdlVersion::Vhdl2008),
            Some(TokenKind::Keyword(KeywordKind::Context))
        );
    }

    #[test]
    fn test_non_keyword() {
        assert_eq!(lookup_keyword("foobar", VhdlVersion::Vhdl2008), None);
    }

    #[test]
    fn test_all_87_keywords_count() {
        let keywords_87 = [
            "abs",
            "access",
            "after",
            "alias",
            "all",
            "and",
            "architecture",
            "array",
            "assert",
            "attribute",
            "begin",
            "block",
            "body",
            "buffer",
            "bus",
            "case",
            "component",
            "configuration",
            "constant",
            "disconnect",
            "downto",
            "else",
            "elsif",
            "end",
            "entity",
            "exit",
            "file",
            "for",
            "function",
            "generate",
            "generic",
            "guarded",
            "if",
            "in",
            "inout",
            "is",
            "label",
            "library",
            "linkage",
            "loop",
            "map",
            "mod",
            "nand",
            "new",
            "next",
            "nor",
            "not",
            "null",
            "of",
            "on",
            "open",
            "or",
            "others",
            "out",
            "package",
            "port",
            "procedure",
            "process",
            "range",
            "record",
            "register",
            "rem",
            "report",
            "return",
            "select",
            "severity",
            "signal",
            "subtype",
            "then",
            "to",
            "transport",
            "type",
            "units",
            "until",
            "use",
            "variable",
            "wait",
            "when",
            "while",
            "with",
            "xor",
        ];
        assert_eq!(keywords_87.len(), 81);
        for kw in &keywords_87 {
            assert!(
                lookup_keyword(kw, VhdlVersion::Vhdl1987).is_some(),
                "Expected '{}' to be a VHDL-1987 keyword",
                kw
            );
        }
    }
}
