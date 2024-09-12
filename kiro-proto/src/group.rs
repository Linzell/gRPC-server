pub mod v1 {
    include!("./group/v1/group.v1.rs");
    #[cfg(feature = "json")]
    include!("./group/v1/group.v1.serde.rs");
}
