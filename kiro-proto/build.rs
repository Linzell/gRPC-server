use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = Path::new("src");
    let proto_dir = Path::new("SRC-Proto/proto");

    std::fs::create_dir_all(out_dir.join("auth/v1")).unwrap();
    std::fs::create_dir_all(out_dir.join("client/v1")).unwrap();
    std::fs::create_dir_all(out_dir.join("common/v1")).unwrap();
    std::fs::create_dir_all(out_dir.join("group/v1")).unwrap();
    std::fs::create_dir_all(out_dir.join("project/v1")).unwrap();

    #[cfg(feature = "json")]
    let well_known_types_path = "::pbjson_types";

    #[cfg(not(feature = "json"))]
    let well_known_types_path = "::prost_types";

    tonic_build::configure()
        .out_dir(out_dir.join("common/v1"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("common/v1").join("proto_descriptor.bin"))
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
        let descriptor_set = std::fs::read(out_dir.join("common/v1").join("proto_descriptor.bin"))?;
        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .ignore_unknown_fields()
            .out_dir(out_dir.join("common/v1"))
            .build(&[".common"])?;
    }

    #[allow(unused_mut)]
    let mut builder = tonic_build::configure()
        .out_dir(out_dir.join("auth/v1"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("auth/v1").join("proto_descriptor.bin"))
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", well_known_types_path)
        .extern_path(".common", "crate::common");

    #[cfg(feature = "postgres")]
    {
        builder = builder.message_attribute("internal.DeviceSession", "#[derive(diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)] #[diesel(sql_type = diesel::sql_types::Binary)]");
    }

    builder.compile(
        &[
            proto_dir
                .join("auth/v1")
                .join("session.proto")
                .to_str()
                .unwrap(),
            proto_dir
                .join("auth/v1")
                .join("auth_service.proto")
                .to_str()
                .unwrap(),
        ],
        &[proto_dir.to_str().unwrap()],
    )?;

    #[cfg(feature = "json")]
    {
        let descriptor_set = std::fs::read(out_dir.join("auth/v1").join("proto_descriptor.bin"))?;
        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .ignore_unknown_fields()
            .out_dir(out_dir.join("auth/v1"))
            .extern_path(".common", "crate::common")
            .build(&[".auth"])?;
    }

    #[allow(unused_mut)]
    let mut builder = tonic_build::configure()
        .out_dir(out_dir.join("client/v1"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("client/v1").join("proto_descriptor.bin"))
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", well_known_types_path)
        .extern_path(".common", "crate::common");

    #[cfg(feature = "postgres")]
    {
        builder = builder.message_attribute("internal.DeviceSession", "#[derive(diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)] #[diesel(sql_type = diesel::sql_types::Binary)]");
    }

    builder.compile(
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
        let descriptor_set = std::fs::read(out_dir.join("client/v1").join("proto_descriptor.bin"))?;
        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .ignore_unknown_fields()
            .out_dir(out_dir.join("client/v1"))
            .extern_path(".common", "crate::common")
            .build(&[".client"])?;
    }

    #[allow(unused_mut)]
    let mut builder = tonic_build::configure()
        .out_dir(out_dir.join("group/v1"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("group/v1").join("proto_descriptor.bin"))
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", well_known_types_path)
        .extern_path(".common", "crate::common");

    #[cfg(feature = "postgres")]
    {
        builder = builder.message_attribute("internal.DeviceSession", "#[derive(diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)] #[diesel(sql_type = diesel::sql_types::Binary)]");
    }

    builder.compile(
        &[
            proto_dir
                .join("group/v1")
                .join("member.proto")
                .to_str()
                .unwrap(),
            proto_dir
                .join("group/v1")
                .join("group.proto")
                .to_str()
                .unwrap(),
            proto_dir
                .join("group/v1")
                .join("group_service.proto")
                .to_str()
                .unwrap(),
        ],
        &[proto_dir.to_str().unwrap()],
    )?;

    #[cfg(feature = "json")]
    {
        let descriptor_set = std::fs::read(out_dir.join("group/v1").join("proto_descriptor.bin"))?;
        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .ignore_unknown_fields()
            .out_dir(out_dir.join("group/v1"))
            .extern_path(".common", "crate::common")
            .build(&[".group"])?;
    }

    Ok(())
}
