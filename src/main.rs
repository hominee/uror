#[macro_use]
extern crate diesel;
mod api;
mod entity;
mod schema;

//#[feature(obfs)]
//pub fn main() {
//api::main().unwrap();
//}

//#[feature(hash)]
pub fn main() {
    api::main().unwrap();
}

//#[cfg(all(not(feature = "obfs"), not(feature = "hash")))]
//compile_error!("To build binary without `obfs` feature, enable `hash` feature instead");
