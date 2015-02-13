#![feature(plugin)]
#![plugin(parallel_macros)]

extern crate parallel;

#[test]
fn empty() {
    execute!();
}

#[test]
fn no_spawn() {
    execute!(|| println!("Hello world!"));
}
