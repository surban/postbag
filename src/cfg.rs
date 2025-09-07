/// Configuration.
pub trait Cfg: Copy + Clone {
    /// Whether struct field identifiers and enum variant identifiers
    /// are serialized.
    fn with_idents() -> bool;
}

/// Configuration.
#[derive(Debug, Clone, Copy)]
pub struct Config<const WITH_IDENTS: bool>;

impl<const WITH_IDENTIFIERS: bool> Cfg for Config<WITH_IDENTIFIERS> {
    fn with_idents() -> bool {
        WITH_IDENTIFIERS
    }
}

/// Serialize with identifiers.
///
/// Struct field identifiers and enum variant identifiers are serialized
/// as strings.
pub type Full = Config<true>;

/// Serialize without identifiers.
///
/// Struct field identifiers are not serialized.
/// Enum variants are serialized using their index.
pub type Slim = Config<false>;
