use std::collections::{HashMap};
use crate::symbol::Symbol;
use crate::grammar::Grammar;

#[derive(Debug, Clone, Eq, PartialEq)]
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
    bookmark: Option<u32>,
    goto: Option<u32>
}

impl<'a> BookmarkedRule<'a>
{
    pub fn new(lhs: Symbol, rhs: &'a Vec<Symbol>) -> BookmarkedRule
    {
        let bookmark = if rhs.is_empty()
        {
            None
        }
        else
        {
            Some(0)
        };

        BookmarkedRule{
            lhs,
            rhs,
            bookmark,
            goto: None
        }
    }
    pub fn advance_dot(&mut self, next_symbol: &Symbol)
    {
        if let Some(index) = self.bookmark
        {
            if self.rhs[index as usize] == *next_symbol
            {
                self.bookmark = 
                    if index < self.rhs.len() as u32 - 1
                    {
                        Some(index + 1)
                    }
                    else
                    {
                        None
                    };
            }
        }
    }

}

impl<'a> std::fmt::Display for BookmarkedRule<'a>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{} ->", self.lhs).unwrap();
        for (index, s) in self.rhs.iter().enumerate()
        {
            if Some(index as u32) == self.bookmark
            {
                write!(f, " ~").unwrap();
            }

            write!(f, " {}", s).unwrap();
        }
        if let Some(goto) = self.goto
        {
            write!(f, " goto {}", goto).unwrap();
        }
        Ok(())
    }
}

struct State<'a>
{
    kernel: BookmarkedRule<'a>,
    closure: Vec<BookmarkedRule<'a>>
}

impl<'a> State<'a>
{
    pub fn advance_dot(&mut self, next_symbol: &Symbol)
    {
        self.kernel.advance_dot(next_symbol);
        for item in self.closure.iter_mut()
        {
            item.advance_dot(next_symbol);
        }
    }
}

impl<'a> std::fmt::Display for State<'a>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        writeln!(f, "==============================\n{}\n------------------------------", self.kernel).unwrap();
        for item in &self.closure
        {
            writeln!(f, " {}", item).unwrap();
        }
        writeln!(f, "==============================").unwrap();
        Ok(())
    }
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
            grammar,
            parse_table,
            stack: Vec::<StackSymbol>::new()
        }
    }

    pub fn parse(&self, program: &Vec<Symbol>) -> Result<(), String>
    {
        let mut handle = Vec::<StackSymbol>::new();
        let mut remaining_input: Vec<Symbol> = program.iter().map(|x| x.clone()).rev().collect::<Vec<Symbol>>();

        while !remaining_input.is_empty()
        {
            let current_state = handle.last().map(|s| s.state).unwrap_or(0);
            let next_token = remaining_input.pop().unwrap();
            let temp = (current_state, next_token);
            let action = &self.parse_table.get(&temp).ok_or( "parse error" )?;
            let (_current_state, next_token) = temp;
            match action
            {
                Action::Shift(state) => {
                    handle.push(
                        StackSymbol
                        {
                            symbol: next_token,
                            state: *state
                        }
                    );
                },
                Action::Reduce( (lhs, rhs) ) => {
                    for item in rhs.iter().rev()
                    {
                        assert_eq!(handle.pop().unwrap().symbol, *item);
                    }
                    
                    remaining_input.push(lhs.clone());
                }
            }
        }


        panic!("Not implemented");
    }

    fn build_table<'b>(grammar: &Grammar) -> HashMap<(u32, Symbol), Action<'a>>
    {
        let parse_table = HashMap::<(u32, Symbol), Action<'a>>::new();
        let all_states = Vec::<State>::new();
        let mut work_list = Vec::<&State>::new();
        work_list.push(&all_states[0]);



        parse_table
    }

}