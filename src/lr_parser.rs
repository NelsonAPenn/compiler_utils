use std::collections::{HashMap, HashSet};
use crate::symbol::Symbol;
use crate::grammar::Grammar;

#[derive(Debug, Clone, Eq, PartialEq)]
struct StackSymbol
{
    symbol: Symbol,
    state: u32
}

#[derive(Debug, Clone, Hash)]
struct BookmarkedRule<'a>
{
    pub lhs: Symbol,
    pub rhs: &'a Vec<Symbol>,

    // the index of the next symbol in the rule to handle, or none if done
    pub bookmark: Option<u32>,
    pub goto: Option<u32>
}

impl<'a> Eq for BookmarkedRule<'a> { }

impl<'a> PartialEq for BookmarkedRule<'a>
{
    fn eq(&self, other: &Self) -> bool
    {
        return self.lhs == other.lhs && self.rhs == other.rhs && self.bookmark == other.bookmark;
    }
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
    // pub fn with_dot_advanced(&self, next_symbol: &Symbol) -> BookmarkedRule<'a>
    // {
    //     let mut clone = self.clone();
    //     if let Some(index) = self.bookmark
    //     {
    //         if self.rhs[index as usize] == *next_symbol
    //         {
    //             clone.bookmark = 
    //                 if index < self.rhs.len() as u32 - 1
    //                 {
    //                     Some(index + 1)
    //                 }
    //                 else
    //                 {
    //                     None
    //                 };
    //         }
    //     }
    //     clone
    // }

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
    pub id: u32,
    pub kernel: BookmarkedRule<'a>,
    pub closure: Vec<BookmarkedRule<'a>>
}

impl<'a> PartialEq for State<'a>
{
    fn eq(&self, other: &Self) -> bool
    {
        return self.kernel == other.kernel;
    }
}

impl<'a> Eq for State<'a> {}

impl<'a> State<'a>
{
    // pub fn with_dot_advanced(&mut self, next_symbol: &Symbol)
    // {
    //     self.kernel.advance_dot(next_symbol);
    //     for item in self.closure.iter_mut()
    //     {
    //         item.advance_dot(next_symbol);
    //     }
    // }
}

impl<'a> std::fmt::Display for State<'a>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        writeln!(f, "==============================\n {}\n------------------------------", self.kernel).unwrap();
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
    all_states: Vec<State<'a>>,
    work_list: Vec<&'a State<'a>>,
    grammar: Grammar,
    parse_table: HashMap<(u32, Symbol), Action<'a> >,
}

impl<'a> LRParser<'a>
{
    pub fn new(grammar: Grammar) -> LRParser<'a>
    {
        let mut parser = LRParser{
            all_states: vec![],
            work_list: vec![], 
            grammar: grammar,
            parse_table: HashMap::<(u32, Symbol), Action<'a>>::new()
        };

        parser.build_table();
        parser
    }

    fn build_state(&'a self, kernel: BookmarkedRule<'a>, id: u32) -> State<'a>
    {
        let closure = self.build_closure(&kernel);
        State
        {
            id,
            kernel,
            closure
        }

    }

    fn build_closure(&'a self, kernel: &BookmarkedRule<'a>) -> Vec<BookmarkedRule<'a>>
    {

        let mut consider_list = HashSet::<BookmarkedRule<'a>>::new();
        consider_list.insert(
            kernel.clone()
        );

        loop 
        {
            let initial_length = consider_list.len().clone();

            let mut new_items = Vec::<BookmarkedRule<'a>>::new();

            for rule_to_consider in consider_list.iter()
            {
                if let Some(bookmark) = rule_to_consider.bookmark
                {
                    let next_symbol = &rule_to_consider.rhs[bookmark as usize];
                    if !next_symbol.terminal
                    {
                        if let Some(productions) = self.grammar.productions.get(next_symbol)
                        {
                            new_items.append(
                                &mut productions
                                .iter()
                                .map(|x| BookmarkedRule::new(next_symbol.clone(), x))
                                .collect::<Vec<BookmarkedRule<'a>>>());
                        }
                    }
                }
            }

            for item in new_items.into_iter()
            {
                consider_list.insert(item);
            }

            if consider_list.len() == initial_length
            {
                break;
            }
        }

        consider_list.retain(|x| x != kernel);
        consider_list.into_iter().collect::<Vec<BookmarkedRule<'a>>>()
    }

    pub fn parse(&'a self, program: &Vec<Symbol>) -> Result<(), String>
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

        Ok(())
    }

    fn add_state(&'a mut self, kernel: BookmarkedRule<'a>)
    {

        let mut potential_new_state = self.build_state(kernel, self.all_states.len() as u32);
        if self.all_states.contains(&potential_new_state)
        {

        }
        else
        {
            self.all_states.push(potential_new_state);
        }

    }

    fn build_table(&mut self)
    {
        let parse_table = HashMap::<(u32, Symbol), Action<'a>>::new();
        let mut all_states = Vec::<State>::new();
        let mut work_list = Vec::<&State>::new();

        // Push Start into known states. 
        let start_symbol = Symbol
        {
            label: String::from("Start"),
            terminal: false
        };
        let rhs = &self.grammar.productions.get(&start_symbol).unwrap()[0];
        let kernel = BookmarkedRule
        {
            lhs: start_symbol,
            rhs,
            bookmark: Some(0),
            goto: None
        };
        all_states.push(self.build_state(kernel, 0));
        work_list.push(&all_states[0]);

        // main part
        while let Some(state) = work_list.pop()
        {

            let rules_to_check = state.closure.iter().chain(vec![&state.kernel].into_iter());

            for rule in rules_to_check
            {
                if let Some(next_symbol) = rule.bookmark.map(|index| &rule.rhs[index as usize])
                {
                    // add command to shift and go to new state
                    // self.parse_table.insert((state.id, next_symbol), Action::Shift());
                }
            }

        }


        self.parse_table = parse_table;
    }

}

#[test]
fn test_closure()
{
        let grammar = Grammar::from_file("data/bnf");
        let parser = LRParser::new(grammar.clone()); 


        let lhs = Symbol
            {
                label: String::from("A"),
                terminal: false
            };

        let kernel = BookmarkedRule
        {
            lhs: lhs.clone(),
            rhs: &grammar.productions.get(&lhs).unwrap()[0],
            bookmark: Some(1),
            goto: None
        };


        println!("{}", parser.build_state(kernel, 0));
}

#[test]
fn test_neverending()
{
        let grammar = Grammar::from_file("data/eeeee");
        let parser = LRParser::new(grammar.clone()); 


        let lhs = Symbol
            {
                label: String::from("E"),
                terminal: false
            };

        let kernel = BookmarkedRule
        {
            lhs: lhs.clone(),
            rhs: &grammar.productions.get(&lhs).unwrap()[0],
            bookmark: Some(0),
            goto: None
        };

        println!("{}", parser.build_state(kernel, 0));

}
