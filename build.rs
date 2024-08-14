use std::io::Result;

fn main() -> Result<()> {
    const COMMON_PROTOS: &[&str] = &["SRC-Proto/proto/common/v1/common.proto"];
    const CLIENT_PROTOS: &[&str] = &[
        "SRC-Proto/proto/client/v1/notifications.proto",
        "SRC-Proto/proto/client/v1/privacy.proto",
        "SRC-Proto/proto/client/v1/security.proto",
        "SRC-Proto/proto/client/v1/settings.proto",
        "SRC-Proto/proto/client/v1/user.proto",
        "SRC-Proto/proto/client/v1/client_service.proto",
    ];

    const INCLUDES: &[&str] = &["SRC-Proto/proto"];

    tonic_build::configure()
        .out_dir("src/api")
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_well_known_types(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .build_client(true)
        .build_server(false)
        .compile(COMMON_PROTOS, INCLUDES)?;

    tonic_build::configure()
        .out_dir("src/api")
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_well_known_types(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .file_descriptor_set_path("src/api/CLIENT_FILE_DESCRIPTOR_SET")
        .build_client(false)
        .build_server(true)
        .compile(CLIENT_PROTOS, INCLUDES)?;

    Ok(())
}
