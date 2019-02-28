//! Library that decodes the binary documents contained on an APK (both resources.arsc and binary
//! XMLs).
//!
//! It exposes also structures to query this binary files on a structured way. For example, it's
//! possible to check which chunks of data a document contains, and perform specific queries
//! depending on the type of chunk.

#![forbid(anonymous_parameters)]
#![deny(
    clippy::all,
    variant_size_differences,
    unused_results,
    unused_qualifications,
    unused_import_braces,
    unsafe_code,
    trivial_numeric_casts,
    trivial_casts,
    missing_docs,
    unused_extern_crates,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic)]
// Allowing these for now:
#![allow(
    missing_docs,
    unused_results,
    clippy::unreadable_literal,
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::similar_names,
    clippy::shadow_unrelated
)]

#[cfg(feature = "zip_decode")]
pub mod apk;
pub mod chunks;
pub mod decoder;
pub mod encoder;
pub mod model;
#[cfg(test)]
pub mod raw_chunks;
#[cfg(test)]
pub mod test;
pub mod visitor;

/// Contents of android's resources.arsc
pub const STR_ARSC: &[u8] = include_bytes!("../resources/resources.arsc");
