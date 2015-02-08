//! (Hopefully) Safe fork-join parallelism abstractions
//!
//! Implements the [`divide`](fn.divide.html) and
//! [`execute!`](../parallel_macros/macro.execute!.html) functions proposed in Niko's
//! [blog](http://smallcultfollowing.com/babysteps/blog/2013/06/11/data-parallelism-in-rust/).
//!
//! # Cargo
//!
//! ``` notrust
//! # Cargo.toml
//! [dependencies.parallel]
//! git = "https://github.com/japaric/parallel.rs"
//!
//! [dependencies.parallel_macros]
//! git = "https://github.com/japaric/parallel.rs"
//! ```

#![allow(unused_features)]
#![deny(warnings)]
#![feature(core)]
#![feature(plugin)]
#![feature(std_misc)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[plugin]
extern crate quickcheck_macros;
#[cfg(test)]
extern crate rand;

use std::mem;
use std::thread::{JoinGuard, Thread};

pub use divide::divide;

mod divide;

trait FnBox<R> {
    fn call_box(self: Box<Self>) -> R;
}

impl<R, F> FnBox<R> for F where F : FnOnce() -> R {
    fn call_box(self: Box<F>) -> R {
        (*self)()
    }
}

/// Spawns an unsafe thread that may outlive its captured references.
///
/// The caller must ensure to call `join()` before the references become invalid.
///
/// **Note** You normally don't want to use this directly because it's unsafe. Instead use the safe
/// [`execute!`](../parallel_macros/macro.execute!.html) macro.
pub unsafe fn fork<'a, T, F>(f: F) -> JoinGuard<'a, T> where T: Send, F: FnOnce() -> T + 'a {
    // XXX Is there any way to avoid passing through a trait object?
    let f = mem::transmute::<_, Box<FnBox<T> + Send>>(Box::new(f) as Box<FnBox<T>>);

    Thread::scoped(move || f.call_box())
}
