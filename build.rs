fn main() {
    #[cfg(feature = "serialization-protobuf")]
    prost_build::compile_protos(&["protobuf/proof.proto"], &["protobuf/"])
        .expect("could not compile protos");
}
