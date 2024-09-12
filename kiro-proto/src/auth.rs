pub mod v1 {
    include!("./auth/v1/auth.v1.rs");
    #[cfg(feature = "json")]
    include!("./auth/v1/auth.v1.serde.rs");
}
