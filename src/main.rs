use serde::{Deserialize, Serialize};
use std::process::Command;
use std::{env, fs, path::Path};
use std::collections::HashMap;

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

// Registry to track installed packages: { "pkgname": "version" }
type Registry = HashMap<String, String>;

#[derive(Deserialize, Debug)]
struct AurResponse { results: Vec<AurPackage> }
#[derive(Deserialize, Debug)]
struct AurPackage {
    #[serde(rename = "Name")] name: String,
    #[serde(rename = "Version")] version: String,
}

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
            for pkg in response.results {
                println!("{:<25} v{}", pkg.name, pkg.version);
            }
        },
        "install" => {
            let pkg_build_dir = format!("{}/{}", config.build_dir, target);
            if !Path::new(&config.build_dir).exists() { fs::create_dir_all(&config.build_dir)?; }

            if Path::new(&pkg_build_dir).exists() { fs::remove_dir_all(&pkg_build_dir)?; }
            let clone_url = format!("https://aur.archlinux.org/{}.git", target);
            Command::new("git").args(&["clone", "--depth", "1", &clone_url, &pkg_build_dir]).status()?;

            if config.show_diff {
                println!(":: Reviewing PKGBUILD...");
                Command::new("git").args(&["-C", &pkg_build_dir, "diff", "PKGBUILD"]).status()?;
            }

            env::set_current_dir(&pkg_build_dir)?;
            let mut build_args = vec!["-sic"];
            if config.noconfirm { build_args.push("--noconfirm"); }
            if !config.check_gpg { build_args.push("--skippgpcheck"); }
            if !config.check_sha256 { build_args.push("--skipchecksums"); }

            if Command::new("makepkg").args(&build_args).status()?.success() {
                println!("==> {} installed successfully!", target);
                update_registry(target.to_string(), "latest".to_string())?;
            }
        },
        _ => println!("Unknown action: {}", action),
    }
    Ok(())
}

fn load_config() -> ApmConfig {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let config_dir = format!("{}/.config/apm", home);
    let yaml_path = format!("{}/apm.yaml", config_dir);
    let reg_path = format!("{}/registry.json", config_dir);

    // Automation: Create directory and default files if missing
    if !Path::new(&config_dir).exists() {
        let _ = fs::create_dir_all(&config_dir);

        let default_yaml = "build_dir: \"/tmp/apm-builds\"\nnoconfirm: true\ncheck_gpg: true\ncheck_sha256: true\nshow_diff: true\n";
        
        // Use printf to generate the YAML
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("printf '{}' > {}", default_yaml, yaml_path))
            .status();

        // Create the empty registry
        let _ = fs::write(&reg_path, "{}");
        println!(":: Initialized apm environment in {}", config_dir);
    }

    let content = fs::read_to_string(&yaml_path).unwrap_or_default();
    serde_yaml::from_str(&content).unwrap_or_else(|_| ApmConfig::default())
}

fn update_registry(name: String, version: String) -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME").expect("HOME not set");
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
