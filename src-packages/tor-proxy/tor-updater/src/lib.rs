use std::{
    env,
    fmt::Display,
    fs::{self, File},
    io::{BufReader, Cursor, Read, Write},
    path::{Path, PathBuf},
    thread,
};

use flate2::bufread::GzDecoder;
use lazy_static::lazy_static;
use openssl::hash::{Hasher, MessageDigest};
use tar::Archive;
use zip::{write::SimpleFileOptions, ZipWriter};

lazy_static! {
    pub static ref DIGEST: MessageDigest = MessageDigest::sha256();
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

/// Read the version from the tor_expert_bundle_version.txt file
fn read_version() -> anyhow::Result<String> {
    // Get the path to the parent directory of the tor-updater crate
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let parent_dir = Path::new(&manifest_dir).parent().unwrap();

    let version_path = parent_dir
        .join("assets")
        .join("tor_expert_bundle_version.txt");

    let mut version = String::new();
    File::open(version_path)?.read_to_string(&mut version)?;
    Ok(version.trim().to_string())
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

pub fn download_version(out_dir: &Path) -> anyhow::Result<()> {
    let out_dir = out_dir.to_path_buf();
    let version = read_version()?;
    let downloads = get_download_info(&version);

    // Downloading archives
    fs::create_dir_all(&out_dir)?;

    let mut handles = Vec::new();
    for download_info in downloads {
        let temp = out_dir.clone();
        let version = version.clone();
        handles.push(thread::spawn(move || process(download_info, version, temp)));
    }

    for h in handles {
        h.join().unwrap()?;
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
            println!(
                "Skipping {} {} with version {} as it is already downloaded",
                os, arch, version
            );
            return Ok(());
        }

        println!(
            "Replacing version {} with {} on os {} {}",
            version_local, version, os, arch
        );
    }

    if download_dir.is_dir() {
        fs::remove_dir_all(&download_dir)?;
    }

    let _ = fs::create_dir_all(&download_dir);

    let out_file = download_dir.join("tor.tar.gz");

    println!("Downloading {} {} from {}", os, arch, download_url);
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

    println!("Unpacking...");

    let out_archive = download_dir.join("unpacked").into_boxed_path();
    fs::create_dir(&out_archive)?;

    let tar_gz = File::open(&out_file)?;
    let tar = GzDecoder::new(BufReader::new(tar_gz));
    let mut archive = Archive::new(tar);

    archive.unpack(&out_archive)?;

    // Find the Tor and Snowflake binaries within the unpacked directory
    println!("Calculating hashes...");

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

    println!("Creating zip...");

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

    println!(
        "Done downloading {} {} with version {}...",
        os, arch, version
    );
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
