// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

fn main() {
    #[cfg(feature = "regenerate")] {
        // We will use self-contained protoc binary, have to hack env param to force it to use
        unsafe {
            std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
        }

        let proto = "proto/dataplane.proto";

        let res = tonic_build::configure()
            .type_attribute(".", "#[derive(::serde::Deserialize, ::serde::Serialize)]")
            .build_server(true)
            .build_client(true)
            .out_dir("src/generated")
            .compile_protos(&[proto], &["proto"]);

        match res {
            Ok(_) => println!("Protobuf compiled successfully!"),
            Err(e) => {
                eprintln!("Failed to compile Protobuf: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(feature = "regenerate"))]
    {
        // Do nothing
    }

}
