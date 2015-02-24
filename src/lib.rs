//! (Hopefully) Safe fork-join parallelism abstractions
//!
//! Implements the [`divide`](fn.divide.html) and
//! [`execute!`](../parallel_macros/macro.execute!.html) functions proposed in Niko's
//! [blog](http://smallcultfollowing.com/babysteps/blog/2013/06/11/data-parallelism-in-rust/).
//!
//! # Cargo
//!
//! ``` text
//! # Cargo.toml
//! [dependencies.parallel]
//! git = "https://github.com/japaric/parallel.rs"
//!
//! [dependencies.parallel_macros]
//! git = "https://github.com/japaric/parallel.rs"
//! ```

#![allow(unused_features)]
#![cfg_attr(test, plugin(quickcheck_macros))]
#![deny(warnings)]
#![feature(core)]
#![feature(os)]
#![feature(plugin)]
#![feature(std_misc)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;

pub use divide::divide;
pub use apply::apply;

mod divide;
mod apply;
