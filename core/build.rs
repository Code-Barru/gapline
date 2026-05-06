fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto = "../proto/gtfs-realtime.proto";
    println!("cargo:rerun-if-changed={proto}");
    let fds = protox::compile([proto], ["../proto"])?;
    prost_build::Config::new()
        .skip_protoc_run()
        .compile_fds(fds)?;
    Ok(())
}
