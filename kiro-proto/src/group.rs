pub mod v1 {
    include!("./group/group.v1.rs");
    #[cfg(feature = "json")]
    include!("./group/group.v1.serde.rs");
}
