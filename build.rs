use std::env;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let ui_path = Path::new(&manifest_dir).join("ui").join("app.slint");
    
    // Ensure the UI file exists
    if !ui_path.exists() {
        panic!("UI file not found at: {}", ui_path.display());
    }
    
    slint_build::compile(ui_path).unwrap();
    
    // Tell Cargo to rerun this build script if the UI file changes
    println!("cargo:rerun-if-changed=ui/app.slint");
}
