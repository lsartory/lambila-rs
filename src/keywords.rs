use crate::token::TokenKind;
use crate::version::VhdlVersion;

/// Look up whether `word` (case-insensitive) is a reserved keyword in the
/// given VHDL version. Returns `Some(TokenKind::Kw_*)` if it is, `None`
/// otherwise.
pub fn lookup_keyword(word: &str, version: VhdlVersion) -> Option<TokenKind> {
    // Convert to lowercase for case-insensitive matching.
    let lower: String = word.to_ascii_lowercase();

    // First, look up the keyword and its minimum required version.
    let (kind, min_version) = match lower.as_str() {
        // ── VHDL-1987 keywords (81) ───────────────────────────────────────
        "abs" => (TokenKind::Kw_Abs, VhdlVersion::Vhdl1987),
        "access" => (TokenKind::Kw_Access, VhdlVersion::Vhdl1987),
        "after" => (TokenKind::Kw_After, VhdlVersion::Vhdl1987),
        "alias" => (TokenKind::Kw_Alias, VhdlVersion::Vhdl1987),
        "all" => (TokenKind::Kw_All, VhdlVersion::Vhdl1987),
        "and" => (TokenKind::Kw_And, VhdlVersion::Vhdl1987),
        "architecture" => (TokenKind::Kw_Architecture, VhdlVersion::Vhdl1987),
        "array" => (TokenKind::Kw_Array, VhdlVersion::Vhdl1987),
        "assert" => (TokenKind::Kw_Assert, VhdlVersion::Vhdl1987),
        "attribute" => (TokenKind::Kw_Attribute, VhdlVersion::Vhdl1987),
        "begin" => (TokenKind::Kw_Begin, VhdlVersion::Vhdl1987),
        "block" => (TokenKind::Kw_Block, VhdlVersion::Vhdl1987),
        "body" => (TokenKind::Kw_Body, VhdlVersion::Vhdl1987),
        "buffer" => (TokenKind::Kw_Buffer, VhdlVersion::Vhdl1987),
        "bus" => (TokenKind::Kw_Bus, VhdlVersion::Vhdl1987),
        "case" => (TokenKind::Kw_Case, VhdlVersion::Vhdl1987),
        "component" => (TokenKind::Kw_Component, VhdlVersion::Vhdl1987),
        "configuration" => (TokenKind::Kw_Configuration, VhdlVersion::Vhdl1987),
        "constant" => (TokenKind::Kw_Constant, VhdlVersion::Vhdl1987),
        "disconnect" => (TokenKind::Kw_Disconnect, VhdlVersion::Vhdl1987),
        "downto" => (TokenKind::Kw_Downto, VhdlVersion::Vhdl1987),
        "else" => (TokenKind::Kw_Else, VhdlVersion::Vhdl1987),
        "elsif" => (TokenKind::Kw_Elsif, VhdlVersion::Vhdl1987),
        "end" => (TokenKind::Kw_End, VhdlVersion::Vhdl1987),
        "entity" => (TokenKind::Kw_Entity, VhdlVersion::Vhdl1987),
        "exit" => (TokenKind::Kw_Exit, VhdlVersion::Vhdl1987),
        "file" => (TokenKind::Kw_File, VhdlVersion::Vhdl1987),
        "for" => (TokenKind::Kw_For, VhdlVersion::Vhdl1987),
        "function" => (TokenKind::Kw_Function, VhdlVersion::Vhdl1987),
        "generate" => (TokenKind::Kw_Generate, VhdlVersion::Vhdl1987),
        "generic" => (TokenKind::Kw_Generic, VhdlVersion::Vhdl1987),
        "guarded" => (TokenKind::Kw_Guarded, VhdlVersion::Vhdl1987),
        "if" => (TokenKind::Kw_If, VhdlVersion::Vhdl1987),
        "in" => (TokenKind::Kw_In, VhdlVersion::Vhdl1987),
        "inout" => (TokenKind::Kw_Inout, VhdlVersion::Vhdl1987),
        "is" => (TokenKind::Kw_Is, VhdlVersion::Vhdl1987),
        "label" => (TokenKind::Kw_Label, VhdlVersion::Vhdl1987),
        "library" => (TokenKind::Kw_Library, VhdlVersion::Vhdl1987),
        "linkage" => (TokenKind::Kw_Linkage, VhdlVersion::Vhdl1987),
        "loop" => (TokenKind::Kw_Loop, VhdlVersion::Vhdl1987),
        "map" => (TokenKind::Kw_Map, VhdlVersion::Vhdl1987),
        "mod" => (TokenKind::Kw_Mod, VhdlVersion::Vhdl1987),
        "nand" => (TokenKind::Kw_Nand, VhdlVersion::Vhdl1987),
        "new" => (TokenKind::Kw_New, VhdlVersion::Vhdl1987),
        "next" => (TokenKind::Kw_Next, VhdlVersion::Vhdl1987),
        "nor" => (TokenKind::Kw_Nor, VhdlVersion::Vhdl1987),
        "not" => (TokenKind::Kw_Not, VhdlVersion::Vhdl1987),
        "null" => (TokenKind::Kw_Null, VhdlVersion::Vhdl1987),
        "of" => (TokenKind::Kw_Of, VhdlVersion::Vhdl1987),
        "on" => (TokenKind::Kw_On, VhdlVersion::Vhdl1987),
        "open" => (TokenKind::Kw_Open, VhdlVersion::Vhdl1987),
        "or" => (TokenKind::Kw_Or, VhdlVersion::Vhdl1987),
        "others" => (TokenKind::Kw_Others, VhdlVersion::Vhdl1987),
        "out" => (TokenKind::Kw_Out, VhdlVersion::Vhdl1987),
        "package" => (TokenKind::Kw_Package, VhdlVersion::Vhdl1987),
        "port" => (TokenKind::Kw_Port, VhdlVersion::Vhdl1987),
        "procedure" => (TokenKind::Kw_Procedure, VhdlVersion::Vhdl1987),
        "process" => (TokenKind::Kw_Process, VhdlVersion::Vhdl1987),
        "range" => (TokenKind::Kw_Range, VhdlVersion::Vhdl1987),
        "record" => (TokenKind::Kw_Record, VhdlVersion::Vhdl1987),
        "register" => (TokenKind::Kw_Register, VhdlVersion::Vhdl1987),
        "rem" => (TokenKind::Kw_Rem, VhdlVersion::Vhdl1987),
        "report" => (TokenKind::Kw_Report, VhdlVersion::Vhdl1987),
        "return" => (TokenKind::Kw_Return, VhdlVersion::Vhdl1987),
        "select" => (TokenKind::Kw_Select, VhdlVersion::Vhdl1987),
        "severity" => (TokenKind::Kw_Severity, VhdlVersion::Vhdl1987),
        "signal" => (TokenKind::Kw_Signal, VhdlVersion::Vhdl1987),
        "subtype" => (TokenKind::Kw_Subtype, VhdlVersion::Vhdl1987),
        "then" => (TokenKind::Kw_Then, VhdlVersion::Vhdl1987),
        "to" => (TokenKind::Kw_To, VhdlVersion::Vhdl1987),
        "transport" => (TokenKind::Kw_Transport, VhdlVersion::Vhdl1987),
        "type" => (TokenKind::Kw_Type, VhdlVersion::Vhdl1987),
        "units" => (TokenKind::Kw_Units, VhdlVersion::Vhdl1987),
        "until" => (TokenKind::Kw_Until, VhdlVersion::Vhdl1987),
        "use" => (TokenKind::Kw_Use, VhdlVersion::Vhdl1987),
        "variable" => (TokenKind::Kw_Variable, VhdlVersion::Vhdl1987),
        "wait" => (TokenKind::Kw_Wait, VhdlVersion::Vhdl1987),
        "when" => (TokenKind::Kw_When, VhdlVersion::Vhdl1987),
        "while" => (TokenKind::Kw_While, VhdlVersion::Vhdl1987),
        "with" => (TokenKind::Kw_With, VhdlVersion::Vhdl1987),
        "xor" => (TokenKind::Kw_Xor, VhdlVersion::Vhdl1987),

        // ── VHDL-1993 additions (+16) ─────────────────────────────────────
        "group" => (TokenKind::Kw_Group, VhdlVersion::Vhdl1993),
        "impure" => (TokenKind::Kw_Impure, VhdlVersion::Vhdl1993),
        "inertial" => (TokenKind::Kw_Inertial, VhdlVersion::Vhdl1993),
        "literal" => (TokenKind::Kw_Literal, VhdlVersion::Vhdl1993),
        "postponed" => (TokenKind::Kw_Postponed, VhdlVersion::Vhdl1993),
        "pure" => (TokenKind::Kw_Pure, VhdlVersion::Vhdl1993),
        "reject" => (TokenKind::Kw_Reject, VhdlVersion::Vhdl1993),
        "rol" => (TokenKind::Kw_Rol, VhdlVersion::Vhdl1993),
        "ror" => (TokenKind::Kw_Ror, VhdlVersion::Vhdl1993),
        "shared" => (TokenKind::Kw_Shared, VhdlVersion::Vhdl1993),
        "sla" => (TokenKind::Kw_Sla, VhdlVersion::Vhdl1993),
        "sll" => (TokenKind::Kw_Sll, VhdlVersion::Vhdl1993),
        "sra" => (TokenKind::Kw_Sra, VhdlVersion::Vhdl1993),
        "srl" => (TokenKind::Kw_Srl, VhdlVersion::Vhdl1993),
        "unaffected" => (TokenKind::Kw_Unaffected, VhdlVersion::Vhdl1993),
        "xnor" => (TokenKind::Kw_Xnor, VhdlVersion::Vhdl1993),

        // ── VHDL-2008 additions (+19) ───────────────────────────────────
        "assume" => (TokenKind::Kw_Assume, VhdlVersion::Vhdl2008),
        "assume_guarantee" => (TokenKind::Kw_AssumeGuarantee, VhdlVersion::Vhdl2008),
        "context" => (TokenKind::Kw_Context, VhdlVersion::Vhdl2008),
        "cover" => (TokenKind::Kw_Cover, VhdlVersion::Vhdl2008),
        "default" => (TokenKind::Kw_Default, VhdlVersion::Vhdl2008),
        "fairness" => (TokenKind::Kw_Fairness, VhdlVersion::Vhdl2008),
        "force" => (TokenKind::Kw_Force, VhdlVersion::Vhdl2008),
        "inherit" => (TokenKind::Kw_Inherit, VhdlVersion::Vhdl2008),
        "parameter" => (TokenKind::Kw_Parameter, VhdlVersion::Vhdl2008),
        "property" => (TokenKind::Kw_Property, VhdlVersion::Vhdl2008),
        "protected" => (TokenKind::Kw_Protected, VhdlVersion::Vhdl2008),
        "release" => (TokenKind::Kw_Release, VhdlVersion::Vhdl2008),
        "restrict" => (TokenKind::Kw_Restrict, VhdlVersion::Vhdl2008),
        "restrict_guarantee" => (TokenKind::Kw_RestrictGuarantee, VhdlVersion::Vhdl2008),
        "sequence" => (TokenKind::Kw_Sequence, VhdlVersion::Vhdl2008),
        "strong" => (TokenKind::Kw_Strong, VhdlVersion::Vhdl2008),
        "vmode" => (TokenKind::Kw_Vmode, VhdlVersion::Vhdl2008),
        "vprop" => (TokenKind::Kw_Vprop, VhdlVersion::Vhdl2008),
        "vunit" => (TokenKind::Kw_Vunit, VhdlVersion::Vhdl2008),

        _ => return None,
    };

    // Only return the keyword if the current version is >= the minimum version.
    if version >= min_version {
        Some(kind)
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
            Some(TokenKind::Kw_Entity)
        );
        assert_eq!(
            lookup_keyword("ENTITY", VhdlVersion::Vhdl1987),
            Some(TokenKind::Kw_Entity)
        );
        assert_eq!(
            lookup_keyword("Entity", VhdlVersion::Vhdl1993),
            Some(TokenKind::Kw_Entity)
        );
    }

    #[test]
    fn test_version_gating_93() {
        // XNOR is a VHDL-1993 keyword, not in VHDL-1987
        assert_eq!(lookup_keyword("xnor", VhdlVersion::Vhdl1987), None);
        assert_eq!(
            lookup_keyword("xnor", VhdlVersion::Vhdl1993),
            Some(TokenKind::Kw_Xnor)
        );
        assert_eq!(
            lookup_keyword("xnor", VhdlVersion::Vhdl2008),
            Some(TokenKind::Kw_Xnor)
        );
    }

    #[test]
    fn test_version_gating_2008() {
        // CONTEXT is a VHDL-2008 keyword
        assert_eq!(lookup_keyword("context", VhdlVersion::Vhdl1987), None);
        assert_eq!(lookup_keyword("context", VhdlVersion::Vhdl1993), None);
        assert_eq!(
            lookup_keyword("context", VhdlVersion::Vhdl2008),
            Some(TokenKind::Kw_Context)
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
