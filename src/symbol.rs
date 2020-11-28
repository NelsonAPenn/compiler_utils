use std::fmt::{Display, Result, Formatter};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Symbol
{
    pub label: String,
    pub terminal: bool
}

impl Display for Symbol
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result 
    {
        write!(f, "{}", &self.label)
    }
}

impl From<String> for Symbol
{
    fn from(text: String) -> Symbol
    {
        Symbol {
            terminal: !text.chars().nth(0).unwrap().is_ascii_uppercase(),
            label: text
        }
    }
}