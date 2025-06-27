fn main() {
    println!("cargo::rerun-if-changed=assets/linux/i686/version.txt");
    println!("cargo::rerun-if-changed=assets/linux/x86_64/version.txt");

    println!("cargo::rerun-if-changed=assets/windows/x86_64/version.txt");
    println!("cargo::rerun-if-changed=assets/windows/i686/version.txt");
}