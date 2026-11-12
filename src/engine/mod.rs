//! Shared bindings to the Yuri's Revenge engine.
//!
//! The modules in this directory are only compiled for Windows. Optional
//! features should use these bindings instead of duplicating engine types,
//! addresses and calling-convention declarations.

pub(crate) mod drawing;
pub(crate) mod hooks;
#[cfg(feature = "dev-console")]
pub(crate) mod house;
