#![no_std]

pub use thisdefmterror_macros::DefmtError;

/**
 * Trait that describe an error that supports printing via defmt and core formating
 */
pub trait DefmtError: core::error::Error + defmt::Format {}

