#![feature(concat_idents)]

mod draw;
mod error;
mod objc;
mod win;

fn main() {
    win::init();

    println!("Bye!");
}
