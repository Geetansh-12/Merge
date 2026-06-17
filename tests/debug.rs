use marked_rs::options::Options;
use marked_rs::lexer::Lexer;

#[test]
fn debug_280() {
    let src = "-\n\n  foo\n";
    let options = Options::default();
    let tokens = Lexer::new(src, &options).tokenize();
    println!("{:#?}", tokens);
    let html = marked_rs::parse(src);
    println!("HTML: {:?}", html);
}
