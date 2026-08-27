fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "proto/homeostat/v1/common.proto",
        "proto/homeostat/v1/source.proto",
        "proto/homeostat/v1/executor.proto",
    ];

    tonic_prost_build::configure().compile_protos(&protos, &["proto"])?;

    for proto in protos {
        println!("cargo:rerun-if-changed={proto}");
    }

    Ok(())
}
