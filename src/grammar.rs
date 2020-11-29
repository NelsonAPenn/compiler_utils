use std::collections::{HashMap, VecDeque, HashSet};
use std::fs::read_to_string;
use crate::symbol::Symbol;

#[derive(Debug, Clone)]
pub struct Grammar
{
    tokens_iter:VecDeque<String>,
    pub productions: HashMap<Symbol, Vec<Vec<Symbol>>>,
    pub nonterminals: HashSet<Symbol>,
    pub terminals: HashSet<Symbol>
}

impl Grammar
{
    pub fn from_file(filename: &str) -> Grammar
    {
        let tokens_iter = read_to_string(filename).unwrap().split_whitespace().map(|x| x.to_string() ).collect();

        let mut grammar = Grammar
        {
            tokens_iter: tokens_iter,
            productions: HashMap::<Symbol, Vec<Vec<Symbol>>>::new(),
            nonterminals: HashSet::<Symbol>::new(),
            terminals: HashSet::<Symbol>::new()
        };

        grammar.parse();

        grammar
    }

    pub fn get_rhs(&self, lhs: &Symbol, rhs_id: u32) -> Option<&Vec<Symbol>>
    {
        self.productions.get(&lhs).map(|list| &list[rhs_id as usize])
    }

    fn parse(&mut self)
    {
        let mut pre_hash_map =  Vec::<(Symbol, Vec<Vec<Symbol>>)>::new();
        
        // collect associated productions before building hashmap
        while !self.tokens_iter.is_empty()
        {
            let (new_lhs, mut new_prod_list) = self.parse_rule();
            let mut found_index: Option<usize> = None;
            for i in 0..pre_hash_map.len()
            {
                if pre_hash_map[i].0 == new_lhs
                {
                    found_index = Some(i);
                }
            }

            if let Some(index) = found_index
            {
                pre_hash_map[index].1.append(&mut new_prod_list);
            }
            else
            {
                pre_hash_map.push( (new_lhs, new_prod_list) );
            }
        }
        let pre_hash_map = pre_hash_map; // freeze

        // build the hashmap
        for (lhs, prod_list) in pre_hash_map.into_iter()
        {
            self.productions.insert( lhs, prod_list );
        }

    }

    fn read_symbol(&mut self) -> Symbol
    {
        let symbol = Symbol::from(self.next());

        if symbol.terminal
        {
            self.terminals.insert(symbol.clone());
        }
        else
        {
            self.nonterminals.insert(symbol.clone());
        }

        symbol
    }

    fn parse_rule(&mut self) -> (Symbol, Vec<Vec<Symbol>>)
    {
        let lhs = self.read_symbol();

        self.expect("->");
        let mut prod_list = Vec::<Vec<Symbol>>::new();
        prod_list.push(self.parse_rhs());

        while self.next_symbol_is("|")
        {
            self.expect("|");

            prod_list.push(self.parse_rhs());
        }
        self.expect(";");

        (lhs, prod_list)
    }

    fn peek(&self) -> Option<&String>
    {
        self.tokens_iter.front()
    }

    fn parse_rhs(&mut self) -> Vec<Symbol>
    {
        // assert_eq!(peek()
        let mut out = Vec::<Symbol>::new();

        while !self.next_symbol_is(";") && !self.next_symbol_is("|")
        {
            out.push(self.read_symbol());
        }

        out
    }

    fn next_symbol_is(&self, expected: &str) -> bool
    {
        self.peek() == Some(&expected.to_string())
    }

    fn next(&mut self) -> String 
    {
        self.tokens_iter.pop_front().unwrap()
    }

    fn expect(&mut self, expected: &str)
    {
        let last = self.next();
        assert_eq!(&last[..], expected);
    }



}