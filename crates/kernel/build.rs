fn main() {
    println!("cargo:rustc-link-arg=-Tcrates/kernel/kernel.ld");
}
