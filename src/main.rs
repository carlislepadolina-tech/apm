use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::{env, fs, path::Path};

// --- Configuration & Registry Structures ---

#[derive(Deserialize, Serialize, Debug)]
struct ApmConfig {
    pub build_dir: String,
    pub noconfirm: bool,
    pub check_gpg: bool,
    pub check_sha256: bool,
    pub show_diff: bool,
}

impl Default for ApmConfig {
    fn default() -> Self {
        Self {
            build_dir: "/tmp/apm-builds".to_string(),
            noconfirm: true,
            check_gpg: true,
            check_sha256: true,
            show_diff: true,
        }
    }
}

// Map of { "package-name": "version" }
type Registry = HashMap<String, String>;

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
    #[serde(rename = "Depends")]
    depends: Option<Vec<String>>,
    #[serde(rename = "MakeDepends")]
    makedepends: Option<Vec<String>>,
}

// --- Logic Engine ---

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config();
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
            install_package(target, &config)?;
        }
        _ => println!("Unknown action: {}", action),
    }

    Ok(())
}

/// The core recursive installer
fn install_package(target: &str, config: &ApmConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Fetch Package Info to get dependencies
    let url = format!("https://aur.archlinux.org/rpc/?v=5&type=info&arg[]={}", target);
    let response: AurResponse = reqwest::blocking::get(url)?.json()?;

    if let Some(pkg) = response.results.first() {
        let mut all_deps = pkg.depends.clone().unwrap_or_default();
        all_deps.extend(pkg.makedepends.clone().unwrap_or_default());

        for dep in all_deps {
            // Clean names like 'libpamac-full>=11.0' into 'libpamac-full'
            let dep_name = dep.split(|c| c == '>' || c == '<' || c == '=').next().unwrap().trim();

            if !is_installed(dep_name) {
                if is_in_aur(dep_name) {
                    println!(":: Resolving AUR dependency: {}...", dep_name);
                    install_package(dep_name, config)?;
                } else {
                    println!(":: System dependency {} will be handled by makepkg/pacman.", dep_name);
                }
            }
        }
    }

    // 2. Setup Build Directory
    let pkg_build_dir = format!("{}/{}", config.build_dir, target);
    if !Path::new(&config.build_dir).exists() {
        fs::create_dir_all(&config.build_dir)?;
    }
    if Path::new(&pkg_build_dir).exists() {
        fs::remove_dir_all(&pkg_build_dir)?;
    }

    // 3. Clone Repository
    println!("==> Cloning {} into {}...", target, pkg_build_dir);
    let clone_url = format!("https://aur.archlinux.org/{}.git", target);
    Command::new("git")
        .args(&["clone", "--depth", "1", &clone_url, &pkg_build_dir])
        .status()?;

    // 4. Audit PKGBUILD if enabled
    if config.show_diff {
        println!(":: Reviewing PKGBUILD for {}...", target);
        Command::new("git")
            .args(&["-C", &pkg_build_dir, "diff", "PKGBUILD"])
            .status()?;
    }

    // 5. Build and Install
    let original_dir = env::current_dir()?;
    env::set_current_dir(&pkg_build_dir)?;

    let mut build_args = vec!["-sic"];
    if config.noconfirm { build_args.push("--noconfirm"); }
    if !config.check_gpg { build_args.push("--skippgpcheck"); }
    if !config.check_sha256 { build_args.push("--skipchecksums"); }

    let status = Command::new("makepkg").args(&build_args).status()?;
    
    // Always return to the original dir so recursion doesn't break pathing
    env::set_current_dir(original_dir)?;

    if status.success() {
        println!("==> {} installed successfully!", target);
        update_registry(target.to_string(), "latest".to_string())?;
    }

    Ok(())
}

// --- Helper Functions ---

fn is_installed(pkg: &str) -> bool {
    Command::new("pacman")
        .args(&["-Qq", pkg])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn is_in_aur(pkg: &str) -> bool {
    let url = format!("https://aur.archlinux.org/rpc/?v=5&type=info&arg[]={}", pkg);
    reqwest::blocking::get(url)
        .and_then(|r| r.json::<AurResponse>())
        .map(|res| !res.results.is_empty())
        .unwrap_or(false)
}

fn load_config() -> ApmConfig {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let config_dir = format!("{}/.config/apm", home);
    let yaml_path = format!("{}/apm.yaml", config_dir);
    let reg_path = format!("{}/registry.json", config_dir);

    if !Path::new(&config_dir).exists() {
        let _ = fs::create_dir_all(&config_dir);

        // Automate YAML generation using printf as requested
        let default_yaml = "\
build_dir: \"/tmp/apm-builds\"
noconfirm: true
check_gpg: true
check_sha256: true
show_diff: true";

        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("printf '{}' > {}", default_yaml, yaml_path))
            .status();

        // Create initial registry.json
        let _ = fs::write(&reg_path, "{}");
        println!(":: Initialized APM environment in {}", config_dir);
    }

    let content = fs::read_to_string(&yaml_path).unwrap_or_default();
    serde_yaml::from_str(&content).unwrap_or_else(|_| ApmConfig::default())
}

fn update_registry(name: String, version: String) -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let reg_path = format!("{}/.config/apm/registry.json", home);

    let mut registry: Registry = if Path::new(&reg_path).exists() {
        let data = fs::read_to_string(&reg_path)?;
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    };

    registry.insert(name, version);
    let json = serde_json::to_string_pretty(&registry)?;
    fs::write(reg_path, json)?;
    Ok(())
}
