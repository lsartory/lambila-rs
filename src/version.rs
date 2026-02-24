/// Supported VHDL language versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VhdlVersion {
    /// IEEE Std 1076-1987
    Vhdl1987,
    /// IEEE Std 1076-1993
    Vhdl1993,
    /// IEEE Std 1076-2008
    Vhdl2008,
}

impl std::fmt::Display for VhdlVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VhdlVersion::Vhdl1987 => write!(f, "VHDL-1987"),
            VhdlVersion::Vhdl1993 => write!(f, "VHDL-1993"),
            VhdlVersion::Vhdl2008 => write!(f, "VHDL-2008"),
        }
    }
}
