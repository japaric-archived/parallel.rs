#![feature(phase, unboxed_closures)]

extern crate parallel;
#[phase(plugin)]
extern crate parallel_macros;

#[test]
fn empty() {
    execute!();
}

#[test]
fn no_spawn() {
    execute!(|:| println!("Hello world!"));
}
