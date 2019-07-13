use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{line_ending, not_line_ending, space0, space1},
    combinator::{map, opt},
    error::{context, make_error, ErrorKind, VerboseError},
    multi::{many0, many1},
    sequence::tuple,
    IResult,
};

use crate::types::*;

// parse_assignment_op maps an operator to an Assignment
fn parse_assignment_op<'a>(i: &'a str) -> IResult<&'a str, Assignment, VerboseError<&'a str>> {
    alt((
        map(tag("="), |_| Assignment::Recursive),
        map(tag("+="), |_| Assignment::Append),
        map(tag("?="), |_| Assignment::Conditional),
        map(tag("!="), |_| Assignment::Shell),
        map(tag(":="), |_| Assignment::Simple),
        map(tag("::="), |_| Assignment::Simple),
    ))(i)
}

// A variable name may be any sequence of characters not containing ‘:’, ‘#’, ‘=’, or whitespace
// I am also including ? and ! as these can conflict with the assignment operator
fn is_variable_name(c: char) -> bool {
    match c {
        ':' => false,
        '#' => false,
        '=' => false,
        '!' => false,
        '?' => false,
        ' ' => false,
        '\t' => false,
        '\r' => false,
        '\n' => false,
        _ => true,
    }
}

// parse_variable_name takes valid variable name characters
fn parse_variable_name<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    take_while(is_variable_name)(i)
}

// Parse Variable
fn parse_variable<'a>(i: &'a str) -> IResult<&'a str, Variable, VerboseError<&'a str>> {
    let (i, foo) = parse_variable_name(i)?;
    let (i, _) = space0(i)?;
    let (i, bar) = parse_assignment_op(i)?;
    let (i, _) = space0(i)?;

    let mut parts: Vec<&'a str> = Vec::new();
    let (i, baz) = not_line_ending(i)?;
    parts.push(baz);

    let mut i = i;
    if baz.ends_with("\\") {
        loop {
            //println!("{}", i);
            let (j, _) = line_ending(i)?;
            //println!("{}", j);
            let (j, _) = space0(j)?;
            //println!("{}", j);
            let (j, p) = not_line_ending(j)?;
            i = j;
            parts.push(p);
            if !p.ends_with("\\") {
                break;
            }
        }
    }
    let (i, _) = many0(line_ending)(i)?;
    Ok((
        i,
        Variable {
            name: foo,
            assignment: bar,
            value: parts,
        },
    ))
}

// The Make Manual says it should be a filename but can include wildcards
// We'll use the cross-section of POSIX and Windows standards here
// Wildcards:
// * gobbles everything. foo.* *bar*
// ? is a single character foo.? ???bar???
// [xyz]* or [a-z] is a list of characters
fn is_target_character(c: char) -> bool {
    match c {
        '#' => false,
        '%' => false,
        ':' => false,
        '|' => false,
        '"' => false,
        '<' => false,
        '>' => false,
        ' ' => false,
        '\t' => false,
        '\r' => false,
        '\n' => false,
        _ => true,
    }
}

fn parse_target_name<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    take_while(is_target_character)(i)
}

fn parse_comment<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    let (i, _) = many1(tuple((tag("#"), not_line_ending, many0(line_ending))))(i)?;
    Ok((i, ""))
}

fn parse_recipe<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    let (i, _) = space1(i)?;
    let (i, recipe) = opt(not_line_ending)(i)?;
    let (i, _) = many0(line_ending)(i)?;
    Ok((
        i,
        match recipe {
            Some(r) => r,
            None => "",
        },
    ))
}

fn parse_recipes<'a>(i: &'a str) -> IResult<&'a str, Vec<&'a str>, VerboseError<&'a str>> {
    many0(parse_recipe)(i)
}

fn parse_target_names<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    let (i, _) = space0(i)?;
    let (i, target) = parse_target_name(i)?;
    let (i, _) = space0(i)?;
    match target {
        "" => Err(nom::Err::Error(make_error(i, ErrorKind::Eof))),
        _ => Ok((i, target)),
    }
}

fn parse_target_list<'a>(i: &'a str) -> IResult<&'a str, Vec<&'a str>, VerboseError<&'a str>> {
    many1(parse_target_names)(i)
}

fn parse_prereqs<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    let (i, _) = space0(i)?;
    let (i, prereq) = parse_target_name(i)?;
    let (i, _) = space0(i)?;
    // handle line breaks in lists of prereqs
    let (i, _) = opt(tuple((tag("\\"), line_ending)))(i)?;
    match prereq {
        "" => Err(nom::Err::Error(make_error(i, ErrorKind::Eof))),
        _ => Ok((i, prereq)),
    }
}

fn parse_prereqs_list<'a>(i: &'a str) -> IResult<&'a str, Vec<&'a str>, VerboseError<&'a str>> {
    many0(parse_prereqs)(i)
}

fn parse_rule<'a>(i: &'a str) -> IResult<&'a str, Rule, VerboseError<&'a str>> {
    let (i, name) = context("target", parse_target_list)(i)?;
    let (i, _) = space0(i)?;
    let (i, _) = context("delimiter", tag(":"))(i)?;
    let (i, _) = space0(i)?;
    let (i, prereqs) = context("prereqs", parse_prereqs_list)(i)?;
    let (i, _) = line_ending(i)?;
    let (i, recipe) = context("recipe", parse_recipes)(i)?;

    Ok((
        i,
        Rule {
            targets: name,
            prerequsities: prereqs,
            recipe: recipe,
        },
    ))
}

pub fn parse_makefile<'a>(i: &'a str) -> IResult<&'a str, Makefile, VerboseError<&'a str>> {
    let mut i = i;
    let mut res = Makefile {
        rules: Vec::new(),
        variables: Vec::new(),
    };
    loop {
        if i.is_empty() {
            break;
        }
        match line_ending::<_, VerboseError<&str>>(i) {
            Ok((j, _)) => i = j,
            Err(_) => match parse_comment(i) {
                Ok((j, _)) => {
                    i = j;
                }
                Err(_) => match parse_variable(i) {
                    Ok((j, o)) => {
                        i = j;
                        res.variables.push(o);
                    }
                    Err(_) => match parse_rule(i) {
                        Ok((j, o)) => {
                            i = j;
                            res.rules.push(o);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    },
                },
            },
        }
    }
    Ok((i, res))
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::error::convert_error;
    use std::fs;

    macro_rules! test_op {
        ($name:ident, $i:expr, $o:expr) => {
            #[test]
            fn $name() {
                let res = parse_assignment_op($i);
                match res {
                    Ok((_, o)) => {
                        assert_eq!($o, o);
                    }
                    Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                        println!("{}", convert_error($i, e));
                        panic!();
                    }
                    Err(nom::Err::Incomplete(_)) => unreachable!(),
                }
            }
        };
    }

    test_op!(test_parse_cond_op, "?=", Assignment::Conditional);
    test_op!(test_parse_simple_op1, ":=", Assignment::Simple);
    test_op!(test_parse_simple_op2, "::=", Assignment::Simple);
    test_op!(test_parse_shell, "!=", Assignment::Shell);
    test_op!(test_parse_recursive_op, "=", Assignment::Recursive);
    test_op!(test_parse_append_op, "+=", Assignment::Append);

    #[test]
    fn test_is_variable_name() {
        if is_variable_name('?') {
            panic!("not a var name")
        }
        if is_variable_name('#') {
            panic!("not a var name")
        }
        if !is_variable_name('2') {
            panic!("it's a valid char")
        }
        if !is_variable_name('a') {
            panic!("it's a valid char")
        }
    }

    #[test]
    fn test_parse_variable_name() {
        assert_eq!(parse_variable_name("bar#"), Ok(("#", "bar")));
        assert_eq!(parse_variable_name("b?ar"), Ok(("?ar", "b")));
        assert_eq!(parse_variable_name("b? ar"), Ok(("? ar", "b")));
    }

    #[test]
    fn test_parse_variable() {
        assert_eq!(
            parse_variable("foo := bar\r\n"),
            Ok((
                "",
                Variable {
                    name: "foo",
                    assignment: Assignment::Simple,
                    value: vec!("bar")
                }
            ))
        );
        assert_eq!(
            parse_variable("foo=\n"),
            Ok((
                "",
                Variable {
                    name: "foo",
                    assignment: Assignment::Recursive,
                    value: vec!("")
                }
            ))
        );
    }

    #[test]
    fn test_parse_multiline_variable() {
        let data = "CFLAGS = $(CDEBUG) -I. -I$(srcdir) $(DEFS) \\\n\t\t-DDEF_AR_FILE=\\\"$(DEF_AR_FILE)\\\" \\\n\t\t-DDEFBLOCKING=$(DEFBLOCKING)\n";
        let res = parse_variable(data);
        match res {
            Ok((i, o)) => {
                assert_eq!(o.name, "CFLAGS");
                assert_eq!(o.assignment, Assignment::Recursive);
                assert_eq!(o.value.len(), 3);
                assert_eq!(i, "");
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    #[test]
    fn test_parse_target_name() {
        assert_eq!(parse_target_name("foo.c"), Ok(("", "foo.c")));
        assert_eq!(parse_target_name("foo.*"), Ok(("", "foo.*")));
        assert_eq!(parse_target_name("$(SRCS)"), Ok(("", "$(SRCS)")));
        assert_eq!(parse_target_name("foo.[abc]??"), Ok(("", "foo.[abc]??")));
        assert_eq!(parse_target_name("/usr/src/foo"), Ok(("", "/usr/src/foo")));
        assert_eq!(
            parse_target_name(".\\file\\foo.ps1"),
            Ok(("", ".\\file\\foo.ps1"))
        );
        assert_eq!(parse_target_name("file<"), Ok(("<", "file")));
    }

    #[test]
    fn test_parse_comment() {
        assert_eq!(parse_comment("#comment\n"), Ok(("", "")));
        assert_eq!(
        parse_comment("# Set this to rtapelib.o unless you defined NO_REMOTE,\n# in which case make it empty.\n"),
         Ok(("",""))
    );
    }

    #[test]
    fn test_parse_recipe_single() {
        let data = "\tshar $(SRCS) $(AUX) | compress\n";
        let res = parse_recipe(data);
        match res {
            Ok((_, o)) => {
                assert_eq!(o, "shar $(SRCS) $(AUX) | compress");
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    #[test]
    fn test_parse_recipe_list() {
        let data = "\tcc -c main.c \\\n\tfoo bar baz\n";
        let res = parse_recipes(data);
        match res {
            Ok((_, o)) => {
                assert_eq!(o.len(), 2);
                assert_eq!(o[0], "cc -c main.c \\");
                assert_eq!(o[1], "foo bar baz");
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    #[test]
    fn test_parse_prereq_literal() {
        assert_eq!(parse_prereqs("main.c"), Ok(("", "main.c")));
        assert_eq!(parse_prereqs(" main.c"), Ok(("", "main.c")));
        assert_eq!(parse_prereqs("main.c "), Ok(("", "main.c")));
        assert_eq!(parse_prereqs(" main.c "), Ok(("", "main.c")));
        assert_eq!(parse_prereqs(" main.c \\\n"), Ok(("", "main.c")));
    }

    #[test]
    fn test_parse_prereqs_no_breaks() {
        let data = "main.o foo.o bar.o baz.o quuz.o\n";
        let res = parse_prereqs_list(data);
        match res {
            Ok((i, o)) => {
                assert_eq!(o.len(), 5);
                assert_eq!(o[0], "main.o");
                assert_eq!(o[4], "quuz.o");
                assert_eq!(i, "\n");
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    #[test]
    fn test_parse_prereqs_list_breaks() {
        let data = "main.o foo.o bar.o \\\n\t\t\t baz.o quuz.o\n";
        let res = parse_prereqs_list(data);
        match res {
            Ok((_, o)) => {
                assert_eq!(o.len(), 5);
                assert_eq!(o[0], "main.o");
                assert_eq!(o[4], "quuz.o");
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    #[test]
    fn test_parse_target_with_prereq() {
        let data = "main.o : main.c defs.h\n\tcc -c main.c\n";
        let res = parse_rule(data);
        match res {
            Ok((_, o)) => {
                assert!(o.targets.contains(&"main.o"));
                assert_eq!(o.prerequsities.len(), 2);
                assert_eq!(o.prerequsities[0], "main.c");
                assert_eq!(o.prerequsities[1], "defs.h");
                assert_eq!(o.recipe.len(), 1);
                assert_eq!(o.recipe[0], "cc -c main.c");
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    #[test]
    fn test_parse_target_no_prereq() {
        let data =
        "clean :\n\trm edit main.o kbd.o command.o display.o \\\n\t\tinsert.o search.o files.o utils.o\n";
        let res = parse_rule(data);
        match res {
            Ok((_, o)) => {
                assert!(o.targets.contains(&"clean"));
                assert_eq!(o.prerequsities.len(), 0);
                assert_eq!(o.recipe.len(), 2);
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    #[test]
    fn test_parse_target_split_prereqs_and_recipe() {
        let data = "edit : main.o kbd.o command.o display.o \\\n\tinsert.o search.o files.o utils.o\n\tcc -o edit main.o kbd.o command.o display.o \\\n\tinsert.o search.o files.o utils.o\n";
        let res = parse_rule(data);
        match res {
            Ok((_, o)) => {
                assert!(o.targets.contains(&"edit"));
                assert!(o.prerequsities.contains(&"main.o"));
                assert!(o.prerequsities.contains(&"utils.o"));
                assert_eq!(o.prerequsities.len(), 8);
                assert_eq!(o.recipe.len(), 2);
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    #[test]
    fn test_parse_phony_target() {
        let data = ".PHONY: all\n";
        let res = parse_rule(data);
        match res {
            Ok((_, o)) => {
                assert!(o.targets.contains(&".PHONY"));
                assert_eq!(o.prerequsities.len(), 1);
                assert_eq!(o.prerequsities[0], "all");
                assert_eq!(o.recipe.len(), 0);
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    #[test]
    fn test_parse_target_names() {
        assert_eq!(parse_target_names("main.c"), Ok(("", "main.c")));
        assert_eq!(parse_target_names(" main.c"), Ok(("", "main.c")));
        assert_eq!(parse_target_names("main.c "), Ok(("", "main.c")));
        assert_eq!(parse_target_names(" main.c "), Ok(("", "main.c")));
        assert_eq!(parse_target_names(" main.c\n :"), Ok(("\n :", "main.c")));
    }

    #[test]
    fn test_parse_target_name_list() {
        assert_eq!(
            parse_target_list("main.c main.o:"),
            Ok((":", vec!["main.c", "main.o"]))
        );
    }

    #[test]
    fn test_parse_makefile_simple() {
        let data = fs::read_to_string("./assets/01-simple.mk").expect("ohnoes");
        let data_s = &data[..];
        let res = parse_makefile(data_s);
        match res {
            Ok((_, o)) => {
                assert_eq!(o.variables.len(), 3);
                assert_eq!(o.rules.len(), 10);
                assert!(o.rules[0].targets.contains(&"edit"));
                assert_eq!(o.rules[0].prerequsities.len(), 8);
                assert!(o.rules[0].prerequsities.contains(&"main.o"));
                assert!(o.rules[0].prerequsities.contains(&"utils.o"));
                assert!(o.rules[1].targets.contains(&"main.o"));
                assert!(o.rules[9].targets.contains(&"clean"));
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data_s, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    #[test]
    fn test_parse_makefile_complex() {
        let data = fs::read_to_string("./assets/02-complex.mk").expect("ohnoes");
        let data_s = &data[..];
        let res = parse_makefile(data_s);
        match res {
            Ok((_, o)) => {
                assert_eq!(o.variables.len(), 23);
                assert_eq!(o.rules.len(), 23);
                assert!(o.rules[0].targets.contains(&".PHONY"));
                assert_eq!(o.rules[0].prerequsities.len(), 1);
                assert!(o.rules[0].prerequsities.contains(&"all"));
                assert!(o.rules[22].targets.contains(&"tar.zoo"));
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("{}", convert_error(data_s, e));
                panic!();
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

}
