#![feature(int_to_from_bytes)]
//! A TIFF6.0 library that helps to deal with tiff files.
//!
//! # Reading
//! The library provides a low-level interface helping to deal with the tree structure and another

const TIFF_LE: u16 = 0x4949;
const TIFF_BE: u16 = 0x4D4D;

extern crate chrono;
#[macro_use]
extern crate error_chain;

mod endian;
mod image;
mod reader;
mod value;
mod writer;

pub use endian::{BE, LE};

pub mod tag;
pub use reader::TIFFReader;
pub use writer::TIFFWriter;

pub use image::baseline::Image;
