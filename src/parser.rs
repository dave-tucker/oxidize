use nom::*;
use nom::character::is_alphanumeric;


// Assignment represents the types of assignment
#[derive(Debug, PartialEq, Clone)]
pub enum Assignment {
  Conditional,
  Simple,
  Recursive,
  Append,
  Shell,
}

// A Variable has a name, assignment type and a value
pub struct Variable<'a> {
  name: &'a [u8],
  assignment: Assignment,
  value: &'a [u8],
}

// A Target has a name, prerequisites and a recipe
#[derive(Debug, PartialEq, Clone)]
pub enum Target {
  Name(String),
  Prerequisites(Vec<String>),
  Recipe(Vec<String>),
}

// parse_assignment_op maps an operator to an Assignment
fn parse_assignment_op(input: &[u8]) -> IResult<&[u8], Assignment> {
  alt!(input,
    map!(tag!("="),|_| Assignment::Recursive) |
    map!(tag!("+="),|_| Assignment::Append) |
    map!(tag!("?="),|_| Assignment::Conditional) |
    map!(tag!("!="),|_| Assignment::Shell) |
    map!(tag!(":="),|_| Assignment::Simple)  |
    map!(tag!("::="),|_| Assignment::Simple))
}

macro_rules! test_op {
  ($name:ident, $i:expr, $o:expr) => {
    #[test]
    fn $name() {
      let data = format!("{} bar", $i);
      let res = parse_assignment_op(data.as_bytes());
      match res {
        Ok((i,o)) => {
          assert_eq!($o, o);
          assert_eq!(b" bar", i);
        }
        Err(e) => {
          println!("{:#?}", e);
          panic!();
        }
      }
    }
  }
}

test_op!(test_parse_cond_op,"?=",Assignment::Conditional);
test_op!(test_parse_simple_op1,":=",Assignment::Simple);
test_op!(test_parse_simple_op2,"::=",Assignment::Simple);
test_op!(test_parse_recursive_op,"=",Assignment::Recursive);
test_op!(test_parse_append_op,"+=",Assignment::Append);

// Line Endings parser
fn line_ending(input: &[u8]) -> IResult<&[u8], &[u8]> {
  alt!(input, tag!("\r\n") | tag!("\n"))
}

// A variable name may be any sequence of characters not containing ‘:’, ‘#’, ‘=’, or whitespace
// I am also including ? and ! as these can conflict with the assignment operator
fn is_variable_name(c: u8) -> bool {
  match c {
    b':' => false,
    b'#' => false,
    b'=' => false,
    b'!' => false,
    b'?' => false,
    b' ' => false,
    b'\t' => false,
    b'\r' => false,
    b'\n' => false,
    _ => true,
  }
}

#[test]
fn test_is_variable_name() {
  if is_variable_name(b'?') {
    panic!("not a var name")
  }
  if is_variable_name(b'#') {
    panic!("not a var name")
  }
  if !is_variable_name(b'2') {
    panic!("it's a valid char")
  }
  if !is_variable_name(b'a') {
    panic!("it's a valid char")
  }
}

// parse_variable_name takes valid variable name characters
fn parse_variable_name(input: &[u8]) -> IResult<&[u8], &[u8]> {
  take_while1!(input, is_variable_name)
}

macro_rules! test_variable_name {
  ($name:ident, $i:expr, $o:expr) => {
    #[test]
    fn $name() {
      let res = parse_variable_name($i);
      match res {
        Ok((i,_)) => {
          assert_eq!(i, $o);
        }
        Err(e) => {
          println!("{:#?}", e);
          panic!();
        }
      }
    }
  }
}

test_variable_name!(test_var_name_1,b"bar#",b"#");
test_variable_name!(test_var_name_2,b"b?ar",b"?ar");
test_variable_name!(test_var_name_3,b"b? ar",b"? ar");

// parse_token captures anything that is alphanumeric
// this is a placeholder as I need to cast a wider net for variable values
fn parse_token(input: &[u8]) -> IResult<&[u8], &[u8]> {
  take_while1!(input, is_alphanumeric)
}

#[test]
fn test_parse_token() {
let data = b"foo=bar\n";
  let res = parse_token(data);
  match res {
    Ok((i,o)) => {
      assert_eq!(o, b"foo");
      assert_eq!(i, b"=bar\n");
    }
    Err(e) => {
      println!("{:#?}", e);
      panic!();
    }
  }
}

// Parse Variable
fn parse_variable(input: &[u8]) -> IResult<&[u8], Variable> {
  do_parse!(
    input,
    foo:    parse_variable_name >>
    bar:    parse_assignment_op >>
    baz:    parse_token >>
            line_ending >>
    ( Variable {
      name: foo,
      assignment: bar,
      value: baz,
    } )
  )
}

#[test]
fn test_parse_variable() {
  let data = "foo=bar\n";
  let res = parse_variable(data.as_bytes());
  match res {
    Ok((_,o)) => {
      assert_eq!(o.name, b"foo");
      assert_eq!(o.assignment, Assignment::Recursive);
      assert_eq!(o.value, b"bar");
    }
    Err(e) => {
      println!("{:#?}", e);
      panic!();
    }
  }
}