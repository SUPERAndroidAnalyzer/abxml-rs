//! Library that decodes the binary documents contained on an APK (both resources.arsc and binary
//! XMLs).
//!
//! It exposes also structures to query this binary files on a structured way. For example, it's
//! possible to check which chunks of data a document contains, and perform specific queries
//! depending on the type of chunk.

// #![forbid(anonymous_parameters, unsafe_code)]
// #![deny(
//     clippy::all,
//     clippy::restriction,
//     variant_size_differences,
//     unused_results,
//     unused,
//     unused_qualifications,
//     unused_import_braces,
//     unused_lifetimes,
//     unreachable_pub,
//     trivial_numeric_casts,
//     trivial_casts,
//     missing_docs,
//     rustdoc,
//     missing_debug_implementations,
//     missing_copy_implementations,
//     deprecated_in_future,
//     meta_variable_misuse,
//     non_ascii_idents,
//     rust_2018_compatibility,
//     rust_2018_idioms,
//     future_incompatible,
//     nonstandard_style,
//     //warnings
// )]
// #![warn(
//     clippy::pedantic,
//     clippy::cargo,
//     clippy::dbg_macro,
//     missing_doc_code_examples
// )]

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
