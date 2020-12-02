use std::collections::HashMap;
use crate::symbol::Symbol;
use crate::grammar::Grammar;

pub struct LLParser
{
    grammar: Grammar,
    // (LHS, next_token) -> rhs_id
    parse_table: HashMap<(Symbol, Symbol), u32> 
}

impl LLParser
{
    pub fn new(grammar: Grammar) -> LLParser
    {
        
        let parse_table = LLParser::build_parse_table(&grammar);

        LLParser{
            grammar,
            parse_table
        }
    }

    fn build_parse_table(grammar: &Grammar) -> HashMap<(Symbol, Symbol), u32>
    {
        let mut out = HashMap::<(Symbol, Symbol), u32>::new();

        for (lhs, prod_list) in &grammar.productions
        {
            for (rhs_id, production) in prod_list.iter().enumerate()
            {
                let mut select_set = grammar.first_of_rhs(production);
                if grammar.rhs_derives_lambda(&production)
                {
                    for symbol in grammar.follow(lhs)
                    {
                        select_set.insert(symbol);
                    }
                }

                for item in select_set
                {
                    let key = (lhs.clone(), item.clone());
                    if out.contains_key(&key)
                    {
                        panic!("Predict set conflict for non-terminal {} with next symbol {}.", lhs, item);
                    }
                    else
                    {
                        out.insert( key, rhs_id as u32 );

                    }
                }

            }

        }

        out
    }

    pub fn parse(&self, program: String) -> Result<(), String>
    {

        let mut stack = Vec::<Symbol>::new();
        let mut remaining_input = program
            .split_whitespace()
            .map(|x| Symbol::from(x.to_string()) )
            .rev()
            .collect::<Vec<Symbol>>();

        stack.push(
            Symbol{
                label: String::from("Start"),
                terminal: false
            }
        );

        while !stack.is_empty()
        {
            for symbol in stack.iter()
            {
                print!("{} ", symbol);
            }
            print!("\t\t\t");
            for symbol in remaining_input.iter().rev()
            {
                print!(" {}", symbol);
            }
            println!("");

            let expected = stack.pop().unwrap();

            let lookahead = remaining_input
                .last()
                .ok_or(format!("Unexpected end of file; {} expected.", expected.label))?;

            if expected.terminal
            {
                let incoming_token = remaining_input.pop().unwrap();
                if incoming_token != expected
                {
                    return Err(format!("Unexpected_token {}; {} expected", incoming_token, expected));
                }
            }
            else
            {
                
                let key = (expected, lookahead.clone());
                let rhs_id = self.parse_table
                    .get(&key)
                    .ok_or(format!("Unexpected token {}; {} expected.", lookahead, key.0))?.clone();
                let (expected, _) = key;

                for symbol in self.grammar.productions.get(&expected).unwrap()[rhs_id as usize].iter().rev()
                {
                    stack.push(symbol.clone());
                }

            }
        }

        Ok(())
    }
}

#[test]
fn test_ll()
{
    let grammar = Grammar::from_file("data/bnf");
    let parser = LLParser::new(grammar.clone()); 

    parser.parse(String::from("a b b d c $")).unwrap();

}