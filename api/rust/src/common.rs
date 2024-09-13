pub mod v1 {
    include!("./common/common.v1.rs");
    #[cfg(feature = "json")]
    include!("./common/common.v1.serde.rs");
}
