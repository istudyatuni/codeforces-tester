const VERSION: &str = "VERSION";

fn main() {
    println!("cargo::rerun-if-env-changed={VERSION}");
    if let Ok(version) = std::env::var(VERSION) {
        println!("cargo::rustc-env=CARGO_PKG_VERSION={version}");
        println!("cargo::warning=setting custom version: {version}");
    }
}
