#![feature(concat_idents)]

mod objc;
mod win;

fn main() {
    win::init();

    println!("Bye!");
}
