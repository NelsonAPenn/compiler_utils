pub mod grammar;
pub mod symbol;
pub mod ll_parser;
pub mod lr_parser;


#[cfg(test)]
mod tests {
    use crate::grammar::Grammar;
    use crate::symbol::Symbol;
    use crate::ll_parser::LLParser;
    use crate::lr_parser::LRParser;

    #[test]
    fn it_works() {
        let grammar = Grammar::from_file("data/bnf");
        println!("{:#?}", grammar);
        let parser = LRParser::new(grammar); 
        let program: Vec::<Symbol> = vec![];
        parser.parse(&program);
    }
}
