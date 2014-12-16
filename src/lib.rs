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

#![deny(warnings)]
#![feature(phase, slicing_syntax, unboxed_closures)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[phase(plugin)]
extern crate quickcheck_macros;

use std::any::Any;
use std::sync::Future;
use std::{mem, task};

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

/// An unsafe task that may outlive its captured references
///
/// **Note** You normally don't want to use this directly because it's unsafe. Instead use the
/// [`execute!`](../parallel_macros/macro.execute!.html) macro.
pub struct Task<T> where T: Send, (Future<Result<T, Box<Any + Send>>>);

impl<T> Task<T> where T: Send {
    /// Spawns a new task that will execute `job`
    ///
    /// This is unsafe because the caller must ensure that the lifetimes of the objects captured by
    /// `job` don't outlive the task.
    pub unsafe fn fork<J>(job: J) -> Task<T> where J: FnOnce() -> T {
        // XXX Is there any way to avoid passing through a trait object?
        let job = mem::transmute::<_, Box<FnBox<T> + Send>>(box job as Box<FnBox<T>>);

        Task(task::try_future(move || job.call_box()))
    }

    /// Waits until the task finishes and yields the return value of it's `job`
    ///
    /// # Panics
    ///
    /// Panics if the underlying task panics
    pub fn join(self) -> T {
        if let Ok(value) = self.0.into_inner() {
            value
        } else {
            panic!()
        }
    }
}
