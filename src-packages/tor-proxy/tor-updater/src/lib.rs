use std::{
    fmt::Display,
    fs::{self, File},
    io::{BufReader, Cursor, Read, Write},
    path::{Path, PathBuf},
};

use flate2::bufread::GzDecoder;
use lazy_static::lazy_static;
use openssl::hash::{Hasher, MessageDigest};
use tar::Archive;
use zip::{write::SimpleFileOptions, ZipWriter};

lazy_static! {
    pub static ref DIGEST: MessageDigest = MessageDigest::sha256();
}

/// Logs a message in the cargo build script format
fn cargo_log(message: &str) {
    println!("cargo:warning={}", message);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Os {
    Windows,
    Linux,
}

impl Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Os::Windows => "windows",
                Os::Linux => "linux",
            }
        )
    }
}

/// Generates download information for the supported platforms
fn get_download_info(version: &str) -> Vec<((Os, String), String)> {
    let base_url = format!(
        "https://archive.torproject.org/tor-package-archive/torbrowser/{}/",
        version
    );
    let mut downloads = Vec::new();

    // Windows x86_64
    downloads.push((
        (Os::Windows, "x86_64".to_string()),
        format!(
            "{}tor-expert-bundle-windows-x86_64-{}.tar.gz",
            base_url, version
        ),
    ));

    // Windows i686
    downloads.push((
        (Os::Windows, "i686".to_string()),
        format!(
            "{}tor-expert-bundle-windows-i686-{}.tar.gz",
            base_url, version
        ),
    ));

    // Linux x86_64
    downloads.push((
        (Os::Linux, "x86_64".to_string()),
        format!(
            "{}tor-expert-bundle-linux-x86_64-{}.tar.gz",
            base_url, version
        ),
    ));

    // Linux i686
    downloads.push((
        (Os::Linux, "i686".to_string()),
        format!(
            "{}tor-expert-bundle-linux-i686-{}.tar.gz",
            base_url, version
        ),
    ));

    downloads
}

pub fn download_version(out_dir: &Path, version: &str) -> anyhow::Result<()> {
    let out_dir = out_dir.to_path_buf();
    let version = version.to_string();

    // Detect current platform
    let current_platform = detect_current_platform()?;
    cargo_log(&format!("Detected platform: {} {}", current_platform.0, current_platform.1));
    
    // Get all download info but filter for current platform
    let downloads = get_download_info(&version);
    let matching_download = downloads
        .into_iter()
        .find(|((os, arch), _)| *os == current_platform.0 && *arch == current_platform.1);
    
    if let Some(download_info) = matching_download {
        // Downloading archive
        fs::create_dir_all(&out_dir)?;
        process(download_info, version, out_dir)?;
    } else {
        return Err(anyhow::anyhow!(
            "No matching download found for current platform: {} {}",
            current_platform.0,
            current_platform.1
        ));
    }

    Ok(())
}

fn process(
    ((os, arch), download_url): ((Os, String), String),
    version: String,
    out_dir: PathBuf,
) -> anyhow::Result<()> {
    let download_dir = out_dir
        .join(format!("{}", os))
        .join(arch.to_lowercase())
        .into_boxed_path();

    let version_file = download_dir.join("version.txt").into_boxed_path();
    if version_file.is_file() {
        let mut version_f = File::open(&version_file)?;
        let mut version_local = String::new();
        version_f.read_to_string(&mut version_local)?;

        if version == version_local {
            cargo_log(&format!(
                "Skipping {} {} with version {} as it is already downloaded",
                os, arch, version
            ));
            return Ok(());
        }

        cargo_log(&format!(
            "Replacing version {} with {} on os {} {}",
            version_local, version, os, arch
        ));
    }

    if download_dir.is_dir() {
        fs::remove_dir_all(&download_dir)?;
    }

    let _ = fs::create_dir_all(&download_dir);

    let out_file = download_dir.join("tor.tar.gz");

    cargo_log(&format!("Downloading {} {} from {}", os, arch, download_url));
    let resp = reqwest::blocking::get(&download_url)?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download: HTTP {}",
            resp.status()
        ));
    }

    let mut file = File::create(&out_file)?;
    let mut content = Cursor::new(resp.bytes()?);
    std::io::copy(&mut content, &mut file)?;

    cargo_log("Unpacking...");

    let out_archive = download_dir.join("unpacked").into_boxed_path();
    fs::create_dir(&out_archive)?;

    let tar_gz = File::open(&out_file)?;
    let tar = GzDecoder::new(BufReader::new(tar_gz));
    let mut archive = Archive::new(tar);

    archive.unpack(&out_archive)?;

    // Find the Tor and Snowflake binaries within the unpacked directory
    cargo_log("Calculating hashes...");

    // For Windows, look in Tor/tor.exe and Tor/PluggableTransports/snowflake-client.exe
    // For Linux, look in tor and tor/pluggable_transports/snowflake-client

    let tor_binary_dir = if os == Os::Windows {
        out_archive.join("Tor")
    } else {
        out_archive.join("tor")
    };

    // Calculate the hash for the Tor binary
    let tor_hash = get_hash(&tor_binary_dir, "tor", os)?;
    let tor_hash_f = download_dir.join("tor.hash");
    File::create(tor_hash_f)?.write_all(tor_hash.as_bytes())?;

    // Calculate the hash for the Snowflake binary
    let snowflake_dir = if os == Os::Windows {
        tor_binary_dir.join("PluggableTransports")
    } else {
        tor_binary_dir.join("pluggable_transports")
    };

    if snowflake_dir.exists() {
        let snow_hash = get_hash(&snowflake_dir, "snowflake-client", os)?;
        let snow_hash_f = download_dir.join("snowflake-client.hash");
        File::create(snow_hash_f)?.write_all(snow_hash.as_bytes())?;
    }

    cargo_log("Creating zip...");

    let path = download_dir.join("tor.zip");
    let file = File::create(&path)?;
    let mut zip = ZipWriter::new(file);

    // Add the entire unpacked directory to the zip
    zip.add_directory_from_path(
        &out_archive.to_path_buf(),
        SimpleFileOptions::default().compression_level(Some(9)),
    )?;

    zip.finish()?;

    // Clean up temporary files
    fs::remove_dir_all(&out_archive)?;
    fs::remove_file(&out_file)?;

    // Write version file
    fs::write(version_file, version.clone())?;

    cargo_log(&format!(
        "Done downloading {} {} with version {}...",
        os, arch, version
    ));
    Ok(())
}

fn get_hash(path: &Path, file_name: &str, os: Os) -> anyhow::Result<String> {
    let mut path = path.to_path_buf();
    match os {
        Os::Windows => path.push(format!("{}.exe", file_name)),
        _ => path.push(file_name),
    }

    let path: Box<Path> = path.into_boxed_path();

    if !path.exists() {
        return Err(anyhow::anyhow!("Binary file not found at {:?}", path));
    }

    let mut hasher = Hasher::new(*DIGEST)?;

    let binary_f = File::open(path)?;
    let mut reader = BufReader::new(binary_f);

    let mut binary = Vec::new();
    reader.read_to_end(&mut binary)?;

    hasher.update(&binary)?;
    let hash = hasher.finish()?;

    Ok(hex::encode(hash))
}

/// Detects the current operating system and architecture
fn detect_current_platform() -> anyhow::Result<(Os, String)> {
    // Detect operating system
    let os = if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else {
        return Err(anyhow::anyhow!("Unsupported operating system"));
    };

    // Detect architecture
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64".to_string()
    } else if cfg!(target_arch = "x86") {
        "i686".to_string()
    } else {
        return Err(anyhow::anyhow!("Unsupported architecture"));
    };

    Ok((os, arch))
}
