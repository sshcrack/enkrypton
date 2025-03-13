use std::{
    fmt::Display,
    fs::{self, File},
    io::{BufReader, Cursor, Read, Write},
    path::{Path, PathBuf},
    thread,
};

use flate2::bufread::GzDecoder;
use itertools::Itertools;
use lazy_static::lazy_static;
use openssl::hash::{Hasher, MessageDigest};
use scraper::{Html, Selector};
use tar::Archive;
use zip::{
    write::SimpleFileOptions,
    ZipWriter,
};
use zip_extensions::ZipWriterExtensions;

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

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1);
    if path.is_none() {
        return Err(anyhow::anyhow!(
            "No path given. (first command line argument)"
        ));
    }

    let out_dir = PathBuf::from(path.unwrap()).into_boxed_path();

    let pairs = get_download_links()?;
    // Downloading archives
    fs::create_dir_all(&out_dir)?;

    let mut handles = Vec::new();
    for pair in pairs {
        let temp = out_dir.clone();
        handles.push(thread::spawn(move || process(pair, temp)));
    }

    for h in handles {
        h.join().unwrap().unwrap();
    }
    Ok(())
}

fn process(
    (info, (version, link)): ((Os, String), (String, String)),
    out_dir: Box<Path>,
) -> anyhow::Result<()> {
    let (os, arch) = info;

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

    println!("Downloading {} {}", os, arch);
    let resp = reqwest::blocking::get(&link)?;
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

    // Removing one file to stay under 25mb
    let unused = out_archive.join("debug");
    if unused.is_dir() {
        fs::remove_dir_all(unused)?;
    }

    println!("Calculating hashes...");

    let tor_binary = out_archive.join("tor").into_boxed_path();
    let tor_hash = get_hash(&tor_binary, "tor", os)?;
    let tor_hash_f = download_dir.join("tor.hash");

    let snow_binary = out_archive
        .join("tor/pluggable_transports")
        .into_boxed_path();

    let snow_hash = get_hash(&snow_binary, "snowflake-client", os)?;
    let snow_hash_f = download_dir.join("snowflake-client.hash");

    File::create(tor_hash_f)?.write_all(tor_hash.as_bytes())?;
    File::create(snow_hash_f)?.write_all(snow_hash.as_bytes())?;

    println!("Creating zip...");

    let path = download_dir.join("tor.zip");
    let file = File::create(&path)?;
    let zip = ZipWriter::new(file);
    zip.create_from_directory_with_options(&out_archive.to_path_buf(), |_p| {
        SimpleFileOptions::default().compression_level(Some(9))
    })?;

    fs::remove_dir_all(&out_archive)?;
    fs::remove_file(&out_file)?;

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
    let mut hasher = Hasher::new(*DIGEST)?;

    let binary_f = File::open(path)?;
    let mut reader = BufReader::new(binary_f);

    let mut binary = Vec::new();
    reader.read_to_end(&mut binary)?;

    hasher.update(&binary)?;
    let hash = hasher.finish()?;

    Ok(hex::encode(hash))
}

fn get_download_links() -> anyhow::Result<Vec<((Os, String), (String, String))>> {
    let resp = reqwest::blocking::get("https://www.torproject.org/de/download/tor/")?;
    println!("Status: {}", resp.status());
    let raw_html = resp.text()?;

    let document = Html::parse_document(&raw_html);
    let row_sel = Selector::parse("table.table:nth-child(3) > tbody:nth-child(2) > tr").unwrap();

    let mut pairs = Vec::new();

    let rows = document.select(&row_sel);
    for row in rows {
        let child = row.children().collect_vec();
        let label = child.get(1).unwrap().children().collect_vec();

        let label = label.get(0);
        if label.is_none() {
            continue;
        }

        let label = label.unwrap().value().as_text().unwrap().to_string();
        let label = label.trim();

        let download = child.get(3).unwrap();
        let download = download.children().collect_vec()[1];

        let version = download
            .first_child()
            .unwrap()
            .value()
            .as_text()
            .unwrap()
            .to_string()
            .split(" ")
            .collect_vec()[0]
            .to_string();

        let download = download.value().as_element().unwrap().attr("href").unwrap();

        let parts = label.split(" ").collect_vec();
        let os = parts[0];
        let mut os_enum = None;

        if os.contains("Linux") {
            os_enum = Some(Os::Linux);
        } else if os.contains("Windows") {
            os_enum = Some(Os::Windows);
        }

        if os_enum.is_none() {
            println!("Skipping unsupported os {}", os);
            continue;
        }

        let os_enum = os_enum.unwrap();

        let arch = parts[1];
        let arch = arch.trim_matches(|c| c == '(' || c == ')');

        pairs.push(((os_enum, arch.to_string()), (version, download.to_string())));
    }

    Ok(pairs)
}
