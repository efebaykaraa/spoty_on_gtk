use std::process::Command;
use std::path::Path;
use std::fs;
use std::io::Result;

pub fn check_compiled_files_exist() -> bool {
    let files = [
        "../svelte/.svelte-kit/output/prerendered/pages/success.html",
        "../svelte/.svelte-kit/output/prerendered/pages/error.html", 
        "../svelte/.svelte-kit/output/prerendered/pages/no-code.html",
        "../svelte/.svelte-kit/output/prerendered/pages/token-error.html"
    ];
    
    files.iter().all(|file| Path::new(file).exists())
}

pub async fn compile_svelte_components() -> Result<()> {
    println!("Running pnpm build in ../svelte directory...");
    
    let output = Command::new("pnpm")
        .args(&["build"])
        .current_dir("../svelte")
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                println!("Successfully built Svelte application");
                copy_prerendered_files().await?;
            } else {
                eprintln!("pnpm build failed: {}", String::from_utf8_lossy(&result.stderr));
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "pnpm build failed"));
            }
        }
        Err(e) => {
            eprintln!("Failed to run pnpm build: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

async fn copy_prerendered_files() -> Result<()> {
    // Ensure templates directory exists
    fs::create_dir_all("templates")?;
    
    let files_to_copy = [
        ("../svelte/.svelte-kit/output/prerendered/pages/success.html", "templates/success.html"),
        ("../svelte/.svelte-kit/output/prerendered/pages/error.html", "templates/error.html"),
        ("../svelte/.svelte-kit/output/prerendered/pages/no-code.html", "templates/no_code.html"),
        ("../svelte/.svelte-kit/output/prerendered/pages/token-error.html", "templates/token_error.html"),
    ];

    for (src, dest) in files_to_copy.iter() {
        if Path::new(src).exists() {
            fs::copy(src, dest)?;
            println!("Copied {} to {}", src, dest);
        } else {
            eprintln!("Error: {} not found after build", src);
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File not found: {}", src)));
        }
    }

    Ok(())
}
