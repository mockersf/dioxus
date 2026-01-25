use other_crate::SharedStruct;
use transitive_crate::transitive_hello;

pub fn using_hello() -> SharedStruct {
    SharedStruct(transitive_hello())
}
