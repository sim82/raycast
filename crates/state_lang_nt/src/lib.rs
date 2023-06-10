pub mod frontent;
pub mod parser;
pub mod util; // }
#[test]
fn test_remove_comments() {
    let mut input: String = r"
        bla
        bla2 // comment
        //comment 2
        bla
    "
    .into();
    util::remove_comments(&mut input);
    println!("{input}");
}
