use std::collections::{HashMap, VecDeque};
use crate::symbol::Symbol;
use crate::grammar::Grammar;

struct StackSymbol
{
    symbol: Symbol,
    state: u32
}

struct BookmarkedRule<'a>
{
    lhs: Symbol,
    rhs: &'a Vec<Symbol>,

    // the index of the next symbol in the rule to handle, or none if done
    bookmark: Option<u32>

}

struct State<'a>
{
    kernel: BookmarkedRule<'a>,
    closure: Vec<BookmarkedRule<'a>>
}

enum Action<'a>
{
    Shift(u32), // Shift (State)
    Reduce( (Symbol, &'a Vec<Symbol>) ) // Reduce (Rule)
}

pub struct LRParser<'a>
{
    grammar: Grammar,
    parse_table: HashMap<(u32, Symbol), Action<'a> >,
    stack: Vec<StackSymbol>
}

impl<'a> LRParser<'a>
{
    pub fn new(grammar: Grammar) -> LRParser<'a>
    {
        let parse_table = LRParser::build_table(&grammar);
        LRParser{
            grammar: grammar,
            parse_table: parse_table ,
            stack: Vec::<StackSymbol>::new()
        }
    }

    fn shift(&mut self, stack: &mut VecDeque<Symbol>, symbol: Symbol)
    {
        panic!("Not implemented");
    }

    fn reduce(&mut self, stack: &mut VecDeque<Symbol>, rule: (Symbol, &'a Vec<Symbol>))
    {
        panic!("Not implemented");
    }

    pub fn parse(&mut self, program: VecDeque<Symbol>)
    {
        panic!("Not implemented");
    }

    fn build_table(grammar: &Grammar) -> HashMap<(u32, Symbol), Action<'a>>
    {
        let parse_table = HashMap::<(u32, Symbol), Action<'a>>::new();

        parse_table
    }

}