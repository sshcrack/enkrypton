use std::env;
use std::path::Path;

fn main() {
    // Re-run build script if the version file changes
    println!("cargo:rerun-if-changed=assets/tor_expert_bundle_version.txt");

    // Only download the bundles if not in docs.rs build environment
    if env::var("DOCS_RS").is_err() {
        // Get output directory for assets
        let out_dir = env::var("OUT_DIR").unwrap();
        let out_path = Path::new(&out_dir);

        // Download the Tor expert bundles
        match tor_updater::download_version(out_path) {
            Ok(_) => println!("Successfully downloaded Tor expert bundles"),
            Err(e) => eprintln!("Failed to download Tor expert bundles: {}", e),
        }
    }
}