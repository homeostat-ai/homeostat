//! Versioned gRPC contracts shared by Homeostat components.

pub mod v1 {
    tonic::include_proto!("homeostat.v1");
}
