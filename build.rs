fn main() {
    println!("cargo::rerun-if-changed=include/prayertimes.c");
    println!("cargo::rerun-if-changed=include/abi_probe.c");
    println!("cargo::rerun-if-changed=../../prayertimes.h");

    cc::Build::new()
        .files(["include/prayertimes.c", "include/abi_probe.c"])
        .compile("prayertimes");
}
