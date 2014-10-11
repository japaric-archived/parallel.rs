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
#![feature(if_let, phase, slicing_syntax, tuple_indexing)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[phase(plugin)]
extern crate quickcheck_macros;

use std::any::Any;
use std::sync::Future;
use std::{mem, raw, task};

pub use divide::divide;

mod divide;

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
        let job = box job as Box<FnOnce<(), T>>;
        let to = mem::transmute::<_, raw::TraitObject>(job);

        Task(task::try_future(proc() {
            let job = mem::transmute::<_, Box<FnOnce<(), T> + Sync>>(to);
            job.call_once(())
        }))
    }

    /// Waits until the task finishes and yields the return value of it's `job`
    ///
    /// # Failure
    ///
    /// Fails if the underlying task fails
    pub fn join(self) -> T {
        if let Ok(value) = self.0.unwrap() {
            value
        } else {
            fail!()
        }
    }
}
