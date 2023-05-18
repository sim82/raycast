pub fn main() {
    state_bc::compiler::compile("states/wl6.st", "src/out.img");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=states/wl6.st");
    println!("cargo:rerun-if-changed=Cargo.lock");
}
