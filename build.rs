pub fn main() {
    state_bc::compiler::compile("states/wl6.st", "src/out.img");
    state_lang_nt::frontent::compile("states/wl6.st2", "src/out2.img");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=states/wl6.st");
    println!("cargo:rerun-if-changed=states/wl6.st2");
    println!("cargo:rerun-if-changed=Cargo.lock");
}
