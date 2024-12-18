use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let proto_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let proto_dir = Path::new(&proto_dir);
    let proto_dir = proto_dir.join("proto");
    let cs_dir = proto_dir.join("kiro");

    std::fs::create_dir_all(out_dir.join("google")).unwrap();
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
        .extern_path(".google.protobuf", "crate::google::protobuf")
        .compile(
            &[cs_dir
                .join("common/v1")
                .join("common.proto")
                .to_str()
                .unwrap()],
            &[
                proto_dir.to_str().unwrap(),
                proto_dir.join("google").to_str().unwrap(),
            ],
        )?;

    #[cfg(feature = "json")]
    {
        let descriptor_set = std::fs::read(out_dir.join("common").join("proto_descriptor_v1.bin"))?;
        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .ignore_unknown_fields()
            .out_dir(out_dir.join("common"))
            .build(&[".common"])?;
    }

    // Google
    tonic_build::configure()
        .out_dir(out_dir.join("google"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("google").join("proto_descriptor_v1.bin"))
        .compile_well_known_types(true)
        .compile(
            &[
                proto_dir
                    .join("google/google/protobuf")
                    .join("any.proto")
                    .to_str()
                    .unwrap(),
                proto_dir
                    .join("google/google/protobuf")
                    .join("descriptor.proto")
                    .to_str()
                    .unwrap(),
                proto_dir
                    .join("google/google/protobuf")
                    .join("empty.proto")
                    .to_str()
                    .unwrap(),
                proto_dir
                    .join("google/google/protobuf")
                    .join("struct.proto")
                    .to_str()
                    .unwrap(),
                proto_dir
                    .join("google/google/protobuf")
                    .join("timestamp.proto")
                    .to_str()
                    .unwrap(),
            ],
            &[
                proto_dir.to_str().unwrap(),
                proto_dir.join("google").to_str().unwrap(),
            ],
        )?;

    #[cfg(feature = "json")]
    {
        let descriptor_set = std::fs::read(out_dir.join("google").join("proto_descriptor_v1.bin"))?;
        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .ignore_unknown_fields()
            .out_dir(out_dir.join("google"))
            .build(&[".google.protobuf"])?;
    }

    #[allow(unused_mut)]
    let mut builder = tonic_build::configure()
        .out_dir(out_dir.join("auth"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("auth").join("proto_descriptor.bin"))
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", well_known_types_path)
        .extern_path(".common", "crate::common");

    #[cfg(feature = "postgres")]
    {
        builder = builder.message_attribute("internal.DeviceSession", "#[derive(diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)] #[diesel(sql_type = diesel::sql_types::Binary)]");
    }

    builder.compile(
        &[
            cs_dir
                .join("auth/v1")
                .join("session.proto")
                .to_str()
                .unwrap(),
            cs_dir
                .join("auth/v1")
                .join("auth_service.proto")
                .to_str()
                .unwrap(),
        ],
        &[
            proto_dir.join("kiro").to_str().unwrap(),
            proto_dir.join("google").to_str().unwrap(),
        ],
    )?;

    #[cfg(feature = "json")]
    {
        let descriptor_set = std::fs::read(out_dir.join("auth").join("proto_descriptor.bin"))?;
        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .ignore_unknown_fields()
            .out_dir(out_dir.join("auth"))
            .extern_path(".common", "crate::common")
            .build(&[".auth"])?;
    }

    #[allow(unused_mut)]
    let mut builder = tonic_build::configure()
        .out_dir(out_dir.join("client"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("client").join("proto_descriptor.bin"))
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", well_known_types_path)
        .extern_path(".common", "crate::common");

    #[cfg(feature = "postgres")]
    {
        builder = builder.message_attribute("internal.DeviceSession", "#[derive(diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)] #[diesel(sql_type = diesel::sql_types::Binary)]");
    }

    builder.compile(
        &[
            cs_dir
                .join("client/v1")
                .join("notifications.proto")
                .to_str()
                .unwrap(),
            cs_dir
                .join("client/v1")
                .join("privacy.proto")
                .to_str()
                .unwrap(),
            cs_dir
                .join("client/v1")
                .join("security.proto")
                .to_str()
                .unwrap(),
            cs_dir
                .join("client/v1")
                .join("settings.proto")
                .to_str()
                .unwrap(),
            cs_dir
                .join("client/v1")
                .join("user.proto")
                .to_str()
                .unwrap(),
            cs_dir
                .join("client/v1")
                .join("client_service.proto")
                .to_str()
                .unwrap(),
        ],
        &[
            proto_dir.join("kiro").to_str().unwrap(),
            proto_dir.join("google").to_str().unwrap(),
        ],
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

    #[allow(unused_mut)]
    let mut builder = tonic_build::configure()
        .out_dir(out_dir.join("group"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("group").join("proto_descriptor.bin"))
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", well_known_types_path)
        .extern_path(".common", "crate::common");

    #[cfg(feature = "postgres")]
    {
        builder = builder.message_attribute("internal.DeviceSession", "#[derive(diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)] #[diesel(sql_type = diesel::sql_types::Binary)]");
    }

    builder.compile(
        &[
            cs_dir
                .join("group/v1")
                .join("member.proto")
                .to_str()
                .unwrap(),
            cs_dir
                .join("group/v1")
                .join("group.proto")
                .to_str()
                .unwrap(),
            cs_dir
                .join("group/v1")
                .join("group_service.proto")
                .to_str()
                .unwrap(),
        ],
        &[
            proto_dir.join("kiro").to_str().unwrap(),
            proto_dir.join("google").to_str().unwrap(),
        ],
    )?;

    #[cfg(feature = "json")]
    {
        let descriptor_set = std::fs::read(out_dir.join("group").join("proto_descriptor.bin"))?;
        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .ignore_unknown_fields()
            .out_dir(out_dir.join("group"))
            .extern_path(".common", "crate::common")
            .build(&[".group"])?;
    }

    Ok(())
}
