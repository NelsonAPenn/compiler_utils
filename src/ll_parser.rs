use std::collections::HashSet;
use crate::symbol::Symbol;
use crate::grammar::Grammar;

pub struct LLParser
{
    grammar: Grammar,
    lambda_deriving_symbols: HashSet<Symbol> 
}

impl LLParser
{
    pub fn new(grammar: Grammar) -> LLParser
    {
        LLParser{
            grammar: grammar,
            lambda_deriving_symbols: HashSet::<Symbol>::new()
        }
    }

    fn generate_lambda_set(&mut self)
    {
        let mut previous_size = 0;
        'prod_list: for (lhs, prod_list) in &self.grammar.productions
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
            for (lhs, prod_list) in &self.grammar.productions
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

    pub fn follow(&self, s: &Symbol, visited: &mut HashSet<Symbol>) -> HashSet<Symbol>
    {
        let mut out = HashSet::<Symbol>::new();
        if visited.contains(s)
        {
            return out;
        }
        visited.insert(s.clone());
        for (lhs, prod_list) in &self.grammar.productions
        {
            for prod in prod_list
            {
                for (index, symbol) in prod.iter().enumerate()
                {
                    if symbol == s
                    {
                        let reduced_rhs = &prod[index+ 1..].to_vec();
                        for first in self.first_of_rhs(reduced_rhs).into_iter()
                        {
                            out.insert(first);
                        }
                        if self.rhs_derives_lambda(reduced_rhs)
                        {
                            for follow in self.follow(lhs, visited ).into_iter()
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
        let mut visited = HashSet::<Symbol>::new();
        'symbol: for symbol in rhs 
        {
            if symbol.terminal
            {
                out.insert(symbol.clone());
                break 'symbol;
            }
            else
            {
                for first in self.first_of_symbol(symbol, &mut visited).into_iter()
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

    pub fn first_of_symbol(&self, s: &Symbol, visited: &mut HashSet<Symbol>) -> HashSet<Symbol>
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

        for rule in &self.grammar.productions[s]
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
                    for first in self.first_of_symbol(symbol, visited).into_iter()
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
