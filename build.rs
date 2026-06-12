fn main() {
    let output = std::process::Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output();
    
    let git_hash = if let Ok(output) = output {
        String::from_utf8(output.stdout).unwrap_or_default().trim().to_string()
    } else {
        "unknown".to_string()
    };
    
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    
    // Rerun build script if git HEAD or refs change
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
}
