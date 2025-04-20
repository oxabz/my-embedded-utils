#![no_std]
/*!
 * A crate implementing utilities for defining error in a similar way to thiserror for the embedded space that use defmt.
 * 
 */


pub use thisdefmterror_macros::DefmtError;

/**
 * Trait that describe an error that supports printing via defmt and core formating.
 * 
 * This crate also implements a derive similar to the crate this error
 */
pub trait DefmtError: core::error::Error + defmt::Format {}

