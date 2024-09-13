pub mod v1 {
    include!("./auth/auth.v1.rs");
    #[cfg(feature = "json")]
    include!("./auth/auth.v1.serde.rs");
}
