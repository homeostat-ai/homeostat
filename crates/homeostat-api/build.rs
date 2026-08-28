fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "proto/homeostat/v1/observation.proto",
        "proto/homeostat/v1/action.proto",
    ];

    prost_build::Config::new().compile_protos(&protos, &["proto"])?;

    for proto in protos {
        println!("cargo:rerun-if-changed={proto}");
    }

    Ok(())
}
