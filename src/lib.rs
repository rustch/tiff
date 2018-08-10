#![feature(int_to_from_bytes)]
//! A TIFF6.0 library that helps to deal with tiff files.
//!
//! # Reading
//! The library provides a low-level interface helping to deal with the tree structure and another

extern crate chrono;
#[macro_use]
extern crate error_chain;

mod endian;
mod ifd;
mod tiff;

pub use endian::{BE, LE};

pub mod tag;
pub use tiff::TIFF;
