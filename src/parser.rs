use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_while},
    combinator::opt,
    combinator::fail,
    IResult, Parser, branch::alt,
};

use crate::values::{LocalPtr, Value, ValuePtr, Local};

struct Scope {
    locals: HashMap<String, LocalPtr>,
}

type Result<'a, T> = IResult<&'a str, T>;

fn ws(input: &str) -> Result<&str> {
    take_while(|c: char| c.is_whitespace())(input)
}

fn parse_int_literal(input: &str) -> Result<ValuePtr> {
    // TODO: Error.
    let (input, chars) = take_while(|c| "0123456789_".contains(c))(input)?;
    let number = chars.parse().unwrap();
    let value = ValuePtr::new(Value::IntLiteral(number));
    Ok((input, value))
}

fn parse_float_literal(input: &str) -> Result<ValuePtr> {
    // TODO: Error.
    // TODO: Make this better.
    let (input, chars) = take_while(|c| "0123456789_.e+-".contains(c))(input)?;
    let number = chars.parse().unwrap();
    let value = ValuePtr::new(Value::FloatLiteral(number));
    Ok((input, value))
}

fn parse_bool_literal(input: &str) -> Result<ValuePtr> {
    let base = tag("TRUE")
        .map(|_| ValuePtr::new(Value::BoolLiteral(true)))
        .parse(input);
    if base.is_ok() {
        base
    } else {
        tag("FALSE")
            .map(|_| ValuePtr::new(Value::BoolLiteral(false)))
            .parse(input)
    }
}

fn parse_identifier_into_value<'b>(
    scope: &'b Scope,
) -> impl for<'a> Fn(&'a str) -> Result<'a, ValuePtr> + 'b {
    move |input| {
        parse_identifier(scope)
            .map(|local| ValuePtr::new(Value::Local(local)))
            .parse(input)
    }
}

fn parse_identifier_text(input: &str) -> Result<&str> {
    take_while(|c: char| c.is_alphabetic() || c.is_numeric() || c == '_')(input)
}

fn parse_identifier<'b>(scope: &'b Scope) -> impl for<'a> Fn(&'a str) -> Result<'a, LocalPtr> + 'b {
    move |input: &str| {
        if !input
            .chars()
            .next()
            .map(char::is_alphabetic)
            .unwrap_or(false)
        {
            fail(input)
        } else {
            let (input, text) =parse_identifier_text(input)?;
            if let Some(local) = scope.locals.get(text) {
                Ok((input, local.ptr_clone()))
            } else {
                fail(input)
            }
        }
    }
}

fn parse_assignment_lhs<'b>(scope: &'b mut Scope) -> impl for<'a> Fn(&'a str) -> Result<'a, LocalPtr> + 'b {
    move |input| {
        let (input, label) = alt((tag("local"), tag("input"), tag("output"), tag("")))(input)?;
        let (input, _) = ws(input)?;
        let (input, name) = parse_identifier_text(input)?;
        let (input, _) = ws(input)?;
        let (input, has_colon) = opt(tag(":"))(input)?;
        let (input, typee) = if has_colon.is_some() {
            let (input, _) = ws(input)?;
            let (input, typee) = parse_basic_expression(scope)(input)?;
            (input, typee)
        } else {
            (input, ValuePtr::new(Value::Any))
        };
        let local = LocalPtr::new(Local {
            compile_time_only: false,
            name: name.to_owned(),
            typee,
        });
        Ok((input, local))
    }
}

fn parse_assignment_statement<'b>(scope: &'b Scope) -> impl for<'a> Fn(&'a str) -> Result<'a, ValuePtr> + 'b {
    let (input, )
}
