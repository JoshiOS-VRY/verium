use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let scrypt2 = manifest_dir
        .join("../../../../../../verium-pool/packages/hashing/src/scrypt2.c");
    if !scrypt2.is_file() {
        panic!(
            "scrypt2.c not found at {} — ensure verium-pool is checked out alongside verium",
            scrypt2.display()
        );
    }
    cc::Build::new()
        .file(&scrypt2)
        .opt_level(2)
        .compile("scrypt2");
    println!("cargo:rerun-if-changed={}", scrypt2.display());
}
