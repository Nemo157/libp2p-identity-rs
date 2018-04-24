extern crate prost_build;

fn main() {
    prost_build::Config::new()
        .enum_fields_as_enums()
        .compile_protos(&["src/data.proto"], &["src/"]).unwrap();
}
