#![feature(concat_idents)]

mod error;
mod objc;
mod win;

fn main() {
    win::init();

    println!("Bye!");
}
