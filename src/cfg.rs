//! Configuration of Postbag serialization data format.

use std::fmt;

/// Configuration trait.
pub trait Cfg {
    /// Whether struct field identifiers and enum variant identifiers
    /// are serialized.
    fn with_idents() -> bool;
}

/// Static (compile-time) configuration.
#[derive(Clone, Copy)]
pub struct StaticCfg<const WITH_IDENTS: bool>;

impl<const WITH_IDENTS: bool> fmt::Debug for StaticCfg<WITH_IDENTS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StaticCfg").field("with_idents", &WITH_IDENTS).finish()
    }
}

impl<const WITH_IDENTS: bool> Cfg for StaticCfg<WITH_IDENTS> {
    fn with_idents() -> bool {
        WITH_IDENTS
    }
}
