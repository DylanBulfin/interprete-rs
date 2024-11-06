extern crate interprete_rs_derive;
use interprete_rs::list_comp;
use interprete_rs_derive::{reverse, reverse_func, test_attr};

#[test_attr]
pub struct TestStruct {
    x: u32,
    y: i32,
}

impl TestStruct {
    fn new(x: u32, y: i32) -> Self {
        Self { x, y }
    }
}

reverse_func!(
    {
        ;("amanaplanacanalpanama")!tnirp
    } () backward nf bup
);

fn main() {
    drawkcab()
}
