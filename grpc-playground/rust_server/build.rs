// build.rs

use tonic_build;

fn main() {
    let proto = "../proto/dataplane.proto";

    let _ = tonic_build::compile_protos(proto);
        tonic_build::configure()
            .build_server(true)
            .build_client(false)
            .out_dir("src/protobuf")
            .compile_protos(&["../proto/dataplane.proto"], &["../proto"])
            .unwrap();
        println!("cargo:rerun-if-changed=../proto/dataplane.proto");
    }

