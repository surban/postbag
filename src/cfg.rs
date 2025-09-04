/// Configuration.
pub trait Cfg: Copy + Clone {
    /// Include identifiers in serialization?
    fn with_identifiers() -> bool;
}

/// Configuration.
#[derive(Debug, Clone, Copy)]
pub struct Config<const WITH_IDENTIFIERS: bool>;

impl<const WITH_IDENTIFIERS: bool> Cfg for Config<WITH_IDENTIFIERS> {
    fn with_identifiers() -> bool {
        WITH_IDENTIFIERS
    }
}

/// Default configuration.
pub type DefaultCfg = Config<true>;
