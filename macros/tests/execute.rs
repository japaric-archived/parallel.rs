#![feature(plugin)]

extern crate parallel;
#[plugin]
extern crate parallel_macros;

#[test]
fn empty() {
    execute!();
}

#[test]
fn no_spawn() {
    execute!(|:| println!("Hello world!"));
}
