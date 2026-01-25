use other_crate::SharedStruct;

pub use other_crate::other_hello;
pub use other_crate::transitive_hello;
pub use using_crate::using_hello;

pub fn hello() -> SharedStruct {
    SharedStruct("hello")
}
