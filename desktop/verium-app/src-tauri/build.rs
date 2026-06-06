use std::env;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let env_paths = [
        Path::new(&manifest_dir).join(".env"),
        Path::new(&manifest_dir).join("../.env"),
        Path::new(&manifest_dir).join("../.env.local"),
    ];
    for path in &env_paths {
        if path.is_file() {
            let _ = dotenvy::from_path(path);
        }
    }

    let anon = env::var("POOL_SUPABASE_ANON_KEY")
        .or_else(|_| env::var("VITE_POOL_SUPABASE_ANON_KEY"))
        .unwrap_or_default();
    let url = env::var("POOL_SUPABASE_URL")
        .or_else(|_| env::var("VITE_POOL_SUPABASE_URL"))
        .unwrap_or_else(|_| "https://pctpdmqxideezyqxvxbj.supabase.co".to_string());

    println!("cargo:rustc-env=POOL_SUPABASE_ANON_KEY={anon}");
    println!("cargo:rustc-env=POOL_SUPABASE_URL={url}");
    println!("cargo:rerun-if-env-changed=POOL_SUPABASE_ANON_KEY");
    println!("cargo:rerun-if-env-changed=VITE_POOL_SUPABASE_ANON_KEY");
    println!("cargo:rerun-if-env-changed=POOL_SUPABASE_URL");
    println!("cargo:rerun-if-env-changed=VITE_POOL_SUPABASE_URL");
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=../.env");
    println!("cargo:rerun-if-changed=../.env.local");

    tauri_build::build()
}
