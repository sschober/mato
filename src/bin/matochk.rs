use std::path::{Path, PathBuf};

fn main() {
    check_groff();
    println!();

    let site_font_devpdf = find_site_font_devpdf();
    let devpdf_dirs = find_devpdf_dirs(site_font_devpdf.as_deref());
    if devpdf_dirs.is_empty() {
        println!("❌ no groff devpdf font directories found");
        std::process::exit(1);
    }
    println!("📂 devpdf directories:");
    for dir in &devpdf_dirs {
        println!("  {}", dir.display());
    }
    match &site_font_devpdf {
        Some(p) => println!("📂 local site-font:  {}", p.display()),
        None => println!("⚠️  could not determine local site-font directory"),
    }
    println!();

    let mut all_ok = true;
    let mut has_warnings = false;
    let (ok, warn) = check_font_family(
        "Minion Pro",
        &["MinionR", "MinionI", "MinionB", "MinionBI"],
        &devpdf_dirs,
        site_font_devpdf.as_deref(),
    );
    all_ok &= ok;
    has_warnings |= warn;
    println!();
    let (ok, warn) = check_font_family(
        "Iosevka Curly Slab",
        &["IosevkaCurlySlabR", "IosevkaCurlySlabI", "IosevkaCurlySlabB", "IosevkaCurlySlabBI"],
        &devpdf_dirs,
        site_font_devpdf.as_deref(),
    );
    all_ok &= ok;
    has_warnings |= warn;
    println!();
    let (ok, warn) = check_font_family(
        "Alegreya",
        &["AlegreyaR", "AlegreyaI", "AlegreyaB", "AlegreyaBI"],
        &devpdf_dirs,
        site_font_devpdf.as_deref(),
    );
    all_ok &= ok;
    has_warnings |= warn;

    println!();
    if all_ok && !has_warnings {
        println!("✅ all fonts ok");
    } else if all_ok {
        println!("⚠️  all fonts found but some are not in local site-font — run mato-install-fonts.sh");
    } else {
        println!("❌ some fonts are missing — run mato-install-fonts.sh to install them");
        std::process::exit(1);
    }
}

fn check_groff() {
    match which("groff") {
        Some(path) => {
            let version = std::process::Command::new(&path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();
            let first_line = version.lines().next().unwrap_or("(unknown version)");
            println!("✅ groff: {first_line}");
            println!("  path: {}", path.display());
        }
        None => {
            println!("❌ groff: NOT FOUND");
            std::process::exit(1);
        }
    }
}

/// Returns (all_found, has_warnings).
/// has_warnings is true when any font is found outside the local site-font dir.
fn check_font_family(
    label: &str,
    fonts: &[&str],
    devpdf_dirs: &[PathBuf],
    site_font_devpdf: Option<&Path>,
) -> (bool, bool) {
    println!("{label}:");
    let mut all_found = true;
    let mut has_warnings = false;
    for &font in fonts {
        let found = devpdf_dirs.iter().find(|dir| dir.join(font).is_file());
        match found {
            Some(dir) => {
                let in_site_font = site_font_devpdf.is_some_and(|sf| sf == dir.as_path());
                if in_site_font {
                    println!("  ✅ {font}  ({})", dir.join(font).display());
                } else {
                    println!(
                        "  ⚠️  {font}  ({}) — not in local site-font, may be wiped on groff upgrade",
                        dir.join(font).display()
                    );
                    has_warnings = true;
                }
            }
            None => {
                println!("  ❌ {font}");
                all_found = false;
            }
        }
    }
    (all_found, has_warnings)
}

/// Finds the local site-font devpdf directory.
/// On macOS: derives the prefix from the groff binary path
///   (e.g. /opt/homebrew/bin/groff → /opt/homebrew/etc/groff/site-font/devpdf).
/// On Linux: uses the conventional /usr/local/share/groff/site-font/devpdf.
fn find_site_font_devpdf() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        let groff = which("groff")?;
        // groff binary is at <prefix>/bin/groff; site-font is at <prefix>/etc/groff/site-font
        let prefix = groff.parent()?.parent()?;
        Some(prefix.join("etc/groff/site-font/devpdf"))
    } else {
        Some(PathBuf::from("/usr/local/share/groff/site-font/devpdf"))
    }
}

/// Returns all devpdf directories found under standard groff font search paths.
/// The local site-font devpdf is always listed first when present.
fn find_devpdf_dirs(site_font_devpdf: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // Prepend site-font so it is checked first (and matched for the ✅/⚠️ distinction)
    if let Some(sf) = site_font_devpdf {
        if sf.is_dir() && !dirs.contains(&sf.to_path_buf()) {
            dirs.push(sf.to_path_buf());
        }
    }

    // Respect GROFF_FONT_PATH if set (colon-separated list of directories)
    if let Ok(env_path) = std::env::var("GROFF_FONT_PATH") {
        for base in env_path.split(':').filter(|s| !s.is_empty()) {
            let p = PathBuf::from(base).join("devpdf");
            if p.is_dir() && !dirs.contains(&p) {
                dirs.push(p);
            }
        }
        if !dirs.is_empty() {
            return dirs;
        }
    }

    let search_roots: &[&str] = &[
        "/usr/local/share/groff/site-font",
        "/usr/local/share/groff",
        "/usr/share/groff",
    ];

    for root in search_roots {
        let root = Path::new(root);
        if !root.is_dir() {
            continue;
        }
        // Direct devpdf child (e.g. site-font/devpdf)
        let direct = root.join("devpdf");
        if direct.is_dir() && !dirs.contains(&direct) {
            dirs.push(direct);
            continue;
        }
        // Versioned subdirectories (e.g. 1.23.0/font/devpdf)
        if let Ok(entries) = std::fs::read_dir(root) {
            let mut versioned: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .map(|p| p.join("font").join("devpdf"))
                .filter(|p| p.is_dir())
                .collect();
            versioned.sort();
            versioned.reverse(); // newest version first
            for p in versioned {
                if !dirs.contains(&p) {
                    dirs.push(p);
                }
            }
        }
    }

    dirs
}

fn which(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH")
        .map(|path_var| {
            std::env::split_paths(&path_var)
                .map(|dir| dir.join(name))
                .find(|p| p.is_file())
        })
        .flatten()
}
