use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = Path::new("src");
    let proto_dir = Path::new("SRC-Proto/proto");

    std::fs::create_dir_all(out_dir.join("auth")).unwrap();
    std::fs::create_dir_all(out_dir.join("client")).unwrap();
    std::fs::create_dir_all(out_dir.join("common")).unwrap();
    std::fs::create_dir_all(out_dir.join("group")).unwrap();
    std::fs::create_dir_all(out_dir.join("project")).unwrap();

    #[cfg(feature = "json")]
    let well_known_types_path = "::pbjson_types";

    #[cfg(not(feature = "json"))]
    let well_known_types_path = "::prost_types";

    tonic_build::configure()
        .out_dir(out_dir.join("common"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("common").join("proto_descriptor.bin"))
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", well_known_types_path)
        .compile(
            &[proto_dir
                .join("common/v1")
                .join("common.proto")
                .to_str()
                .unwrap()],
            &[proto_dir.to_str().unwrap()],
        )?;

    #[cfg(feature = "json")]
    {
        let descriptor_set = std::fs::read(out_dir.join("common").join("proto_descriptor.bin"))?;
        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .ignore_unknown_fields()
            .out_dir(out_dir.join("common"))
            .build(&[".common"])?;
    }

    tonic_build::configure()
        .out_dir(out_dir.join("client"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("client").join("proto_descriptor.bin"))
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", well_known_types_path)
        .extern_path(".common", "crate::common")
        .compile(
            &[
                proto_dir
                    .join("client/v1")
                    .join("notifications.proto")
                    .to_str()
                    .unwrap(),
                proto_dir
                    .join("client/v1")
                    .join("privacy.proto")
                    .to_str()
                    .unwrap(),
                proto_dir
                    .join("client/v1")
                    .join("security.proto")
                    .to_str()
                    .unwrap(),
                proto_dir
                    .join("client/v1")
                    .join("settings.proto")
                    .to_str()
                    .unwrap(),
                proto_dir
                    .join("client/v1")
                    .join("user.proto")
                    .to_str()
                    .unwrap(),
                proto_dir
                    .join("client/v1")
                    .join("client_service.proto")
                    .to_str()
                    .unwrap(),
            ],
            &[proto_dir.to_str().unwrap()],
        )?;

    #[cfg(feature = "json")]
    {
        let descriptor_set = std::fs::read(out_dir.join("client").join("proto_descriptor.bin"))?;
        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .ignore_unknown_fields()
            .out_dir(out_dir.join("client"))
            .extern_path(".common", "crate::common")
            .build(&[".client"])?;
    }

    Ok(())
}
