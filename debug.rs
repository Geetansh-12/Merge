use marked_rs::options::Options;
use marked_rs::lexer::Lexer;

fn main() {
    let src = "-\n\n  foo\n";
    let options = Options::default();
    let tokens = Lexer::new(src, &options).tokenize();
    println!("{:#?}", tokens);
}
