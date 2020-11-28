mod grammar;
mod symbol;
mod ll_parser;
mod lr_parser;

use grammar::Grammar;
use ll_parser::LLParser;
use lr_parser::LRParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let grammar = Grammar::from_file("data/bnf");
        println!("{:#?}", grammar);
        let parser = LRParser::new(grammar); 
    }
}
