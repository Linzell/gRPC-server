pub mod v1 {
    include!("./client/client.v1.rs");
    #[cfg(feature = "json")]
    include!("./client/client.v1.serde.rs");
}

// #[cfg(feature = "surrealdb")]
// impl From<crate::client::v1::surrealdb::Error> for tonic::Status {
//     fn from(e: crate::client::v1::surrealdb::Error) -> Self {
//         tonic::Status::new(tonic::Code::Internal, format!("{:?}", e))
//     }
// }
