// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "regenerate")]
fn add_type_generators(bld: tonic_build::Builder, types: &[&str]) -> tonic_build::Builder {
    types.iter().fold(bld, |bld, t| {
        bld.type_attribute(
            t,
            "#[cfg_attr(feature = \"bolero\", derive(::bolero::TypeGenerator))]",
        )
    })
}

fn main() {
    #[cfg(feature = "regenerate")]
    {
        // We will use self-contained protoc binary, have to hack env param to force it to use
        unsafe {
            std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
        }

        let proto = "proto/dataplane.proto";

        let bld = tonic_build::configure();
        let bld = add_type_generators(
            bld,
            &[
                "BgpAddressFamilyIPv4",
                "BgpAddressFamilyIPv6",
                "BgpAddressFamilyL2vpnEvpn",
                "BgpAF",
                "IfType",
                "IfRole",
                "LogLevel",
                "OspfNetworkType",
                "PacketDriver",
            ],
        );
        let res = bld
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
