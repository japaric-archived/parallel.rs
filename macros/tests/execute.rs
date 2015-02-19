#![feature(plugin)]
#![plugin(parallel_macros)]

#[test]
fn empty() {
    execute!();
}

#[test]
fn no_spawn() {
    execute!(|| println!("Hello world!"));
}
