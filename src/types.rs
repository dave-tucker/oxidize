// Assignment represents the types of assignment
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Assignment {
    Conditional,
    Simple,
    Recursive,
    Append,
    Shell,
}

// A Variable has a name, assignment type and a value
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Variable<'a> {
    pub name: &'a str,
    pub assignment: Assignment,
    pub value: &'a str,
}

// A Rule contains a list of targets, prerequisites and the recipe to build them
#[derive(Debug, PartialEq, Clone)]
pub struct Rule<'a> {
    pub targets: Vec<&'a str>,
    pub prerequsities: Vec<&'a str>,
    pub recipe: Vec<String>,
}

// Makefile represents the contents of the file
#[derive(Debug, PartialEq, Clone)]
pub struct Makefile<'a> {
    pub variables: Vec<Variable<'a>>,
    pub rules: Vec<Rule<'a>>,
}
