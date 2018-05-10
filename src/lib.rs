//! Library that decodes the binary documents contained on an APK (both resources.arsc and binary
//! XMLs).
//!
//! It exposes also structures to query this binary files on a structured way. For example, it's
//! possible to check which chunks of data a document contains, and perform specific queries
//! depending on the type of chunk.

#![recursion_limit = "1024"]
#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
#![forbid(anonymous_parameters)]
#![cfg_attr(feature = "cargo-clippy", warn(clippy_pedantic))]
#![deny(variant_size_differences, unused_results, unused_qualifications, unused_import_braces,
        unsafe_code, trivial_numeric_casts, trivial_casts, missing_docs, unused_extern_crates,
        missing_debug_implementations, missing_copy_implementations)]
// Allowing these for now:
#![allow(missing_docs, unused_results)]
#![cfg_attr(feature = "cargo-clippy",
            allow(unreadable_literal, stutter, similar_names, cast_possible_truncation,
                  cast_precision_loss))]

extern crate byteorder;
extern crate encoding;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate xml;
#[cfg(feature = "zip_decode")]
extern crate zip;

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
