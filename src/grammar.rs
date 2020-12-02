use std::collections::{HashMap, VecDeque, HashSet};
use std::fs::read_to_string;
use crate::symbol::Symbol;

#[derive(Debug, Clone)]
pub struct Grammar
{
    tokens_iter:VecDeque<String>,
    pub productions: HashMap<Symbol, Vec<Vec<Symbol>>>,
    pub nonterminals: HashSet<Symbol>,
    pub terminals: HashSet<Symbol>,
    pub lambda_deriving_symbols: HashSet<Symbol> 
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
            terminals: HashSet::<Symbol>::new(),
            lambda_deriving_symbols: HashSet::<Symbol>::new()
        };
        grammar.parse();
        grammar.generate_lambda_set();

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

    fn generate_lambda_set(&mut self)
    {
        let mut previous_size = 0;
        'prod_list: for (lhs, prod_list) in &self.productions
        {
            for prod in prod_list
            {
                if prod.is_empty()
                {
                    self.lambda_deriving_symbols.insert(lhs.clone());
                    continue 'prod_list;
                }
            }
        }
        while self.lambda_deriving_symbols.len() != previous_size
        {
            previous_size = self.lambda_deriving_symbols.len().clone();
            for (lhs, prod_list) in &self.productions
            {
                for prod in prod_list
                {
                    if self.rhs_derives_lambda(prod)
                    {
                        self.lambda_deriving_symbols.insert(lhs.clone());
                    }
                }
            }

        }
    }
    pub fn rhs_derives_lambda(&self, rhs: &Vec<Symbol>) -> bool
    {
        for symbol in rhs
        {
            if !self.lambda_deriving_symbols.contains(&symbol)
            {
                return false;
            }
        }
        true
    }

    pub fn follow(&self, s: &Symbol) -> HashSet<Symbol>
    {
        let mut visited = HashSet::<Symbol>::new();
        self.inner_follow(s, &mut visited)
    }

    fn inner_follow(&self, s: &Symbol, visited: &mut HashSet<Symbol>) -> HashSet<Symbol>
    {
        let mut out = HashSet::<Symbol>::new();
        if visited.contains(s)
        {
            return out;
        }
        visited.insert(s.clone());
        for (lhs, prod_list) in &self.productions
        {
            for prod in prod_list
            {
                for (index, symbol) in prod.iter().enumerate()
                {
                    if symbol == s
                    {
                        let reduced_rhs = &prod[index + 1..].to_vec();
                        for first in self.first_of_rhs(reduced_rhs).into_iter()
                        {
                            out.insert(first);
                        }
                        if self.rhs_derives_lambda(reduced_rhs)
                        {
                            for follow in self.inner_follow(lhs, visited ).into_iter()
                            {
                                out.insert(follow);
                            }
                        }

                    }

                }

            }
        }
        out
    }

    pub fn first_of_rhs(&self, rhs: &Vec<Symbol>) -> HashSet<Symbol>
    {
        let mut out = HashSet::<Symbol>::new();
        'symbol: for symbol in rhs 
        {
            if symbol.terminal
            {
                out.insert(symbol.clone());
                break 'symbol;
            }
            else
            {
                for first in self.first_of_symbol(symbol).into_iter()
                {
                    out.insert(first);
                }
                if !self.lambda_deriving_symbols.contains(symbol)
                {
                    break 'symbol;
                }
            }
        }
        out
    }

    pub fn first_of_symbol(&self, s: &Symbol) -> HashSet<Symbol>
    {
        let mut visited = HashSet::<Symbol>::new();
        self.inner_first_of_symbol(s, &mut visited)
    }

    fn inner_first_of_symbol(&self, s: &Symbol, visited: &mut HashSet<Symbol>) -> HashSet<Symbol>
    {
        let mut out = HashSet::<Symbol>::new();
        if visited.contains(s)
        {
            return out;
        }
        visited.insert(s.clone());
        if s.terminal
        {
            out.insert(s.clone());
            return out;
        }

        for rule in &self.productions[s]
        {
            'symbol: for symbol in rule
            {
                if symbol.terminal
                {
                    out.insert(symbol.clone());
                    break 'symbol;
                }
                else
                {
                    for first in self.inner_first_of_symbol(symbol, visited).into_iter()
                    {
                        out.insert(first);
                    }
                    if !self.lambda_deriving_symbols.contains(symbol)
                    {
                        break 'symbol;
                    }
                }
            }
        }
        out
    }


}