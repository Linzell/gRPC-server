pub mod v1 {
    include!("./common/v1/common.v1.rs");
    #[cfg(feature = "json")]
    include!("./common/v1/common.v1.serde.rs");
}
