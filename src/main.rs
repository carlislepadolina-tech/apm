use serde::Deserialize;
use std::process::Command;
use std::{env, fs, path::Path};

#[derive(Deserialize, Debug)]
struct AurResponse {
    results: Vec<AurPackage>,
}

#[derive(Deserialize, Debug)]
struct AurPackage {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Version")]
    version: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("APM - Archuser Package Manager");
        println!("Usage: apm <search|install> <package>");
        return Ok(());
    }

    let action = &args[1];
    let target = &args[2];

    match action.as_str() {
        "search" => {
            let url = format!("https://aur.archlinux.org/rpc/?v=5&type=search&arg={}", target);
            let response: AurResponse = reqwest::blocking::get(url)?.json()?;
            
            println!("Found {} results:", response.results.len());
            for pkg in response.results {
                println!("{:<25} v{}", pkg.name, pkg.version);
            }
        }
        "install" => {
            // 1. Setup a clean build environment in /tmp
            let build_root = "/tmp/apm-builds";
            let pkg_build_dir = format!("{}/{}", build_root, target);
            
            if !Path::new(build_root).exists() {
                fs::create_dir_all(build_root)?;
            }

            // Wipe old build if it exists to avoid 'git clone' errors
            if Path::new(&pkg_build_dir).exists() {
                fs::remove_dir_all(&pkg_build_dir)?;
            }

            // 2. Clone the AUR repository
            let clone_url = format!("https://aur.archlinux.org/{}.git", target);
            println!("==> Cloning {} into {}...", target, pkg_build_dir);
            
            let clone_status = Command::new("git")
                .args(&["clone", "--depth", "1", &clone_url, &pkg_build_dir])
                .status()?;

            if !clone_status.success() {
                eprintln!("Error: Failed to clone repository.");
                return Ok(());
            }

            // 3. Run makepkg
            println!("==> Starting build for {}...", target);
            env::set_current_dir(&pkg_build_dir)?;
            
            // -s: Sync dependencies (runs pacman)
            // -i: Install package after build
            // -c: Clean up work files after build
            // --noconfirm: Optional, but good for 'yay'-like behavior
            let build_status = Command::new("makepkg")
                .args(&["-sic", "--noconfirm"])
                .status()?;

            if build_status.success() {
                println!("==> {} installed successfully!", target);
            }
        }
        _ => println!("Unknown action: {}", action),
    }

    Ok(())
}
