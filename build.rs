fn main() {
    println!("cargo::rerun-if-changed=include/prayertimes.c");
    println!("cargo::rerun-if-changed=include/abi_probe.c");
    println!("cargo::rerun-if-changed=include/prayertimes.h");
    println!("cargo::rerun-if-changed=include/timezone.c");
    println!("cargo::rerun-if-changed=include/timezone.h");

    cc::Build::new()
        .files([
            "include/prayertimes.c",
            "include/abi_probe.c",
            "include/timezone.c",
        ])
        // abi_probe.c relies on _Static_assert and _Alignof, so the C11
        // baseline has to be explicit rather than left to the compiler default.
        .std("c11")
        .compile("prayertimes");
}
