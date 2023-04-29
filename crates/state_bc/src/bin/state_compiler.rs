fn main() {
    #[cfg(feature = "compiler")]
    {
        let filename = std::env::args().nth(1).expect("missing input file");
        let outname = std::env::args().nth(2).expect("missing output file");
        state_bc::compiler::compile(&filename, &outname);
    }
}
