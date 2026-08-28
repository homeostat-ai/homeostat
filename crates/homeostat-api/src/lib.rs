//! Versioned data contracts shared by Homeostat components.

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/homeostat.v1.rs"));
}
