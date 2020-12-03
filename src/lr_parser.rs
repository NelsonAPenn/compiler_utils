use std::collections::{HashMap, HashSet};
use crate::symbol::Symbol;
use crate::grammar::Grammar;

pub enum Mode
{
    LR0,
    SLR
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct StackSymbol
{
    symbol: Symbol,
    state: u32
}

impl std::fmt::Display for StackSymbol
{
    fn fmt(&self, f: &'_ mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "[{} {}]", self.symbol, self.state)
    }
}

#[derive(Debug, Clone, Hash, PartialOrd, Ord)]
struct BookmarkedRule
{
    pub lhs: Symbol,
    pub rhs_id: u32,

    // the index of the next symbol in the rule to handle, or none if done
    pub bookmark: Option<u32>,
    pub goto: Option<u32>
}

impl Eq for BookmarkedRule { }

impl PartialEq for BookmarkedRule
{
    fn eq(&self, other: &Self) -> bool
    {
        return self.lhs == other.lhs && self.rhs_id == other.rhs_id && self.bookmark == other.bookmark;
    }
}

impl BookmarkedRule
{
    #[allow(dead_code)]
    fn print(&self, grammar: &Grammar)
    {
        print!("{} ->", self.lhs);
        for (index, s) in grammar.get_rhs(&self.lhs, self.rhs_id).unwrap().iter().enumerate()
        {
            if Some(index as u32) == self.bookmark
            {
                print!(" ~");
            }

            print!(" {}", s);
        }
        if let None = self.bookmark
        {
            print!(" ~");
        }
        if let Some(goto) = self.goto
        {
            print!("    goto {}", goto);
        }

    }

}

struct State
{
    pub id: u32,
    pub kernel: Vec<BookmarkedRule>,
    pub closure: Vec<BookmarkedRule>
}

impl PartialEq for State
{
    fn eq(&self, other: &Self) -> bool
    {
        return self.kernel == other.kernel;
    }
}

impl Eq for State {}

impl State
{
    #[allow(dead_code)]
    fn print(&self, grammar: &Grammar)
    {
        print!("\n==============================\n");
        for item in &self.kernel
        {
            print!(" ");
            item.print(&grammar);
            println!("");
        }
        println!("------------------------------");
        for item in &self.closure
        {
            print!(" ");
            item.print(&grammar);
            println!("");
        }
        println!("==============================");
    }
}


#[derive(Debug)]
enum Action
{
    Shift(u32), // Shift (State)
    Reduce( (Symbol, u32) ), // Reduce (Rule)
    Accept
}

pub struct LRParser
{
    grammar: Grammar,
    parse_table: HashMap<(u32, Option<Symbol>), Action >,
    mode: Mode
}

impl LRParser
{
    pub fn new(grammar: Grammar, mode: Mode) -> LRParser
    {
        let mut parser = LRParser{
            grammar: grammar,
            parse_table: HashMap::<(u32, Option<Symbol>), Action>::new(),
            mode
        };

        parser.build_table();
        parser
    }

    fn get_rhs(&self, lhs: &Symbol, rhs_id: u32) -> Option<&Vec<Symbol>>
    {
        self.grammar.get_rhs(lhs, rhs_id)
    }

    fn build_bookmarked_rule(&self, lhs: Symbol, rhs_id: u32) -> BookmarkedRule
    {
        let bookmark = if self.get_rhs(&lhs, rhs_id).unwrap().is_empty()
        {
            None
        }
        else
        {
            Some(0)
        };

        BookmarkedRule{
            lhs,
            rhs_id,
            bookmark,
            goto: None
        }
    }

    fn build_state(&self, kernel: Vec<BookmarkedRule>, id: u32) -> State
    {
        let closure = self.build_closure(&kernel);
        State
        {
            id,
            kernel,
            closure
        }

    }

    fn build_closure(&self, kernel: &Vec<BookmarkedRule>) -> Vec<BookmarkedRule>
    {

        let mut consider_list = kernel
            .clone()
            .into_iter()
            .collect::<HashSet<BookmarkedRule>>();

        loop 
        {
            let initial_length = consider_list.len().clone();

            let mut new_items = Vec::<BookmarkedRule>::new();

            for rule_to_consider in consider_list.iter()
            {
                if let Some(bookmark) = rule_to_consider.bookmark
                {
                    let next_symbol = &self.get_rhs(&rule_to_consider.lhs, rule_to_consider.rhs_id).unwrap()[bookmark as usize];
                    if !next_symbol.terminal
                    {
                        if let Some(productions) = self.grammar.productions.get(next_symbol)
                        {
                            new_items.append(
                                &mut (0..productions.len()).map(
                                    |index|
                                    self.build_bookmarked_rule(next_symbol.clone(), index as u32)
                                ).collect::<Vec<BookmarkedRule>>());
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

        consider_list.retain(|x| !kernel.contains(x));
        consider_list.into_iter().collect::<Vec<BookmarkedRule>>()
    }

    pub fn parse(&self, program: String) -> Result<(), String>
    {

        let mut handle = Vec::<StackSymbol>::new();
        let mut remaining_input = program
            .split_whitespace()
            .map(|x| Symbol::from(x.to_string()) )
            .rev()
            .collect::<Vec<Symbol>>();

        'parse: loop
        {
            print!("handle:");
            for stack_symbol in handle.iter()
            {
                print!(" {}", stack_symbol);
            }
            print!("\nremaining_input: ");
            for symbol in remaining_input.iter().rev()
            {
                print!(" {}", symbol);
            }
            println!("\n");

            let current_state = handle.last().map(|s| s.state).unwrap_or(0);
            let next_token = remaining_input.pop();
            let temp = (current_state, next_token);
            let action = &self.parse_table.get(&temp).ok_or( "parse error" )?;
            let (_current_state, next_token) = temp;
            match action
            {
                Action::Shift(state) => {
                    handle.push(
                        StackSymbol
                        {
                            symbol: next_token.unwrap(),
                            state: *state
                        }
                    );
                },
                Action::Reduce( (lhs, rhs_id) ) => {
                    for item in self.get_rhs(lhs, *rhs_id).unwrap().iter().rev()
                    {
                        assert_eq!(handle.pop().unwrap().symbol, *item);
                    }
                    
                    if let Some(next_token) = next_token
                    {
                        remaining_input.push(next_token);
                    }
                    remaining_input.push(lhs.clone());
                },
                Action::Accept => {
                    break 'parse
                }
            }
        }

        Ok(())
    }

    fn add_state<'b>(&self, all_states:&mut Vec<State>, work_list: &mut Vec<u32>, kernel: Vec<BookmarkedRule>) -> u32 
    {

        let potential_new_state = self.build_state(kernel, all_states.len() as u32);
        if let Some(position) = all_states.iter().position( |state| *state == potential_new_state )
        {
            return position as u32;
        }
        else
        {
            let result = potential_new_state.id;
            all_states.push(potential_new_state);
            work_list.push(result);
            return result;
        }

    }

    fn build_table(&mut self)
    {
        let mut all_states = Vec::<State>::new();
        let mut work_list = Vec::<u32>::new();
        let mut error_messages = Vec::<String>::new();

        // Push Start into known states. 
        let start_symbol = Symbol
        {
            label: String::from("Start"),
            terminal: false
        };
        let kernel = vec![BookmarkedRule
        {
            lhs: start_symbol.clone(),
            rhs_id: 0,
            bookmark: Some(0),
            goto: None
        }];
        all_states.push(self.build_state(kernel, 0));
        work_list.push(0);

        // SHIFTS 
        while let Some(state_id) = work_list.pop()
        {

            let rules_to_check = all_states[state_id as usize].closure.iter().chain(all_states[state_id as usize].kernel.iter());
            
            let mut next_symbols = vec![];

            for rule in rules_to_check.clone()
            {
                if let Some(next_symbol) = rule.bookmark.map(|index| &self.get_rhs(&rule.lhs, rule.rhs_id).unwrap()[index as usize])
                {
                    next_symbols.push(next_symbol.clone());
                }
            }

            next_symbols.sort();
            next_symbols.dedup();

            let mut new_kernels = next_symbols
                .iter()
                .map(|_x| (Vec::<u32>::new(), Vec::<BookmarkedRule>::new()))
                .collect::<Vec<(Vec<u32>, Vec<BookmarkedRule>)>>();

            for (rule_index, rule) in rules_to_check.enumerate()
            {
                let rhs = &self.get_rhs(&rule.lhs, rule.rhs_id).unwrap();
                if let Some(next_symbol) = rule.bookmark.map(|index| &rhs[index as usize])
                {
                    if let Some(index) = next_symbols.iter().position(|symbol| symbol == next_symbol)
                    {
                        let new_bookmark = 
                        if let Some(index) = rule.bookmark{
                            if index == (rhs.len() as u32) - 1
                            {
                                None
                            }
                            else
                            {
                                Some(index + 1 as u32)
                            }

                        }
                        else
                        {
                            None
                        };
                        new_kernels[index as usize].0.push(
                            rule_index as u32
                        );
                        new_kernels[index as usize].1.push(BookmarkedRule{
                            lhs: rule.lhs.clone(),
                            rhs_id: rule.rhs_id,
                            bookmark: new_bookmark,
                            goto: None
                        });
                    }
                }
            }

            for (kernel_index, (indices, mut kernel)) in new_kernels.into_iter().enumerate()
            {
                kernel.sort();
                kernel.dedup();

                let new_state_id = self.add_state(&mut all_states, &mut work_list, kernel);

                for index in indices
                {
                    let old_state = &mut all_states[state_id as usize];
                    old_state.closure.iter_mut().chain(old_state.kernel.iter_mut()).nth(index as usize).unwrap().goto = Some(new_state_id);
                }
                self.parse_table.insert( (state_id, Some(next_symbols[kernel_index].clone())), Action::Shift(new_state_id));
            }
        }

        for (index, state) in all_states.iter().enumerate()
        {
            println!("\nState: {}", index);
            state.print(&self.grammar);
        }

        // REDUCES
        self.parse_table.insert( (0, Some(start_symbol)), Action::Accept);

        for state in all_states.iter()
        {
            let rules_to_check = state.closure.iter().chain(state.kernel.iter());

            for rule in rules_to_check{
                if let None = rule.bookmark{

                    let reduce_set = match &self.mode
                    {
                        Mode::LR0 => {
                            self.grammar.terminals.iter()
                                .chain(
                                    self.grammar.nonterminals.iter().filter(|x| x.label != "Start")
                                )
                                .map(|symbol| Some(symbol.clone()))
                                .chain(vec![None].into_iter())
                                .collect::<HashSet<Option<Symbol>>>()
                        },
                        Mode::SLR => {
                            self.grammar.follow(&rule.lhs).into_iter()
                                .map(|symbol| Some(symbol))
                                .chain(vec![None].into_iter())
                                .collect::<HashSet<Option<Symbol>>>()
                        }
                    };


                    for symbol in reduce_set
                    {
                        let table_tuple = (state.id, symbol);
                        if let Some(action) = self.parse_table.get( &table_tuple )
                        {
                            let (_, symbol) = table_tuple;
                            match action
                            {
                                Action::Shift(_next_state) => {
                                    error_messages.push(format!("Shift-reduce conflict at state {} with symbol {:?}.", state.id, symbol));
                                },
                                Action::Reduce(_rule_id) => {
                                    error_messages.push(format!("Reduce-reduce conflict at state {} with symbol {:?}.", state.id, symbol));
                                },
                                Action::Accept => {
                                    error_messages.push(format!("What in the world!? State {}, symbol {:?}", state.id, symbol));
                                }
                            }
                        }
                        else
                        {
                            self.parse_table.insert( table_tuple, Action::Reduce( (rule.lhs.clone(), rule.rhs_id) ));
                        }
                    }
                }
           }
        }

        if !error_messages.is_empty()
        {
            let mut error_string = String::from("\n");
            for message in error_messages
            {
                error_string += &message[..];
                error_string += "\n";
            }

            panic!(error_string);
        }


    }


}

#[should_panic]
#[test]
fn test_lr0_failure()
{
        let grammar = Grammar::from_file("data/bnf");
        let parser = LRParser::new(grammar.clone(), Mode::LR0); 


        let lhs = Symbol
            {
                label: String::from("A"),
                terminal: false
            };

        let kernel = vec![BookmarkedRule
        {
            lhs: lhs.clone(),
            rhs_id: 0,
            bookmark: Some(1),
            goto: None
        }];


        parser.build_state(kernel, 0).print(&grammar);
}

#[test]
fn test_slr_success()
{
    let grammar = Grammar::from_file("data/bnf");
    let parser = LRParser::new(grammar.clone(), Mode::SLR); 


    for (key, value) in parser.parse_table.iter()
    {
        if let Some(symbol) = &key.1
        {
            println!("({}, {}): {:?}", key.0, symbol, value);
        }
        else
        {
            println!("({}, None): {:?}", key.0, value);

        }
    }

    parser.parse(String::from("a b b d c $")).unwrap();

}

#[should_panic]
#[test]
fn multiple_conflicts_reported()
{
    let grammar = Grammar::from_file("data/10a");
    let _parser = LRParser::new(grammar, Mode::LR0);
}

#[test]
fn test_state_building()
{
    let grammar = Grammar::from_file("data/eeeee");
    let parser = LRParser::new(grammar.clone(), Mode::LR0); 

    for (key, value) in parser.parse_table.iter()
    {
        if let Some(symbol) = &key.1
        {
            println!("({}, {}): {:?}", key.0, symbol, value);
        }
        else
        {
            println!("({}, None): {:?}", key.0, value);

        }
    }

    parser.parse(String::from("plus plus num num num $")).unwrap();


}
#[test]
fn test_neverending()
{
        let grammar = Grammar::from_file("data/self_referencing");
        let parser = LRParser::new(grammar.clone(), Mode::LR0); 


        let lhs = Symbol
            {
                label: String::from("E"),
                terminal: false
            };

        let kernel = vec![BookmarkedRule
        {
            lhs: lhs.clone(),
            rhs_id: 0,
            bookmark: Some(0),
            goto: None
        }];

        parser.build_state(kernel, 0).print(&grammar);

}
