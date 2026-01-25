pub struct SharedStruct(pub &'static str);

pub use transitive_crate::transitive_hello;

pub fn other_hello() -> SharedStruct {
    SharedStruct("hello")
}
