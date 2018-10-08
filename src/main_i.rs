mod processor;

#[macro_use]
extern crate lazy_static;
extern crate regex;

fn main() {
    processor::execute(true)
}
