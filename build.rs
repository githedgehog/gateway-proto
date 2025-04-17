// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

fn main() {
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