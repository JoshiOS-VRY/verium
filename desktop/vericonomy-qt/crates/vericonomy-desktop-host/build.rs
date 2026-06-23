use std::env;

fn main() {
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
}
