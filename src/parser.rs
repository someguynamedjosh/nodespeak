use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    combinator::{fail, opt},
    multi::many0,
    sequence::{terminated, tuple},
    IResult, InputIter, Parser,
};

use crate::values::{Local, LocalPtr, Value, ValuePtr};

#[derive(Clone, Debug)]
pub struct Scope {
    all_locals: HashMap<String, LocalPtr>,
    inputs: Vec<LocalPtr>,
    outputs: Vec<LocalPtr>,
    plain_locals: Vec<LocalPtr>,
}

impl Scope {
    fn new() -> Self {
        Self {
            all_locals: HashMap::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            plain_locals: Vec::new(),
        }
    }
}

type Result<'a, T> = IResult<&'a str, T>;

pub fn parse_body(input: &str) -> Result<(Scope, Vec<ValuePtr>)> {
    let (input, _) = ws(input)?;
    let mut scope = Scope::new();
    let (input, values) = many0(terminated(
        parse_statement(&mut scope),
        tuple((ws, tag(";"), ws)),
    ))(input)?;
    if input.len() > 0 {
        fail(input)
    } else {
        Ok((
            input,
            (scope, values.into_iter().filter_map(|x| x).collect()),
        ))
    }
}

fn ws(input: &str) -> Result<&str> {
    take_while(|c: char| c.is_whitespace())(input)
}

fn parse_int_literal(input: &str) -> Result<ValuePtr> {
    // TODO: Error.
    let (input, chars) = take_while1(|c| "0123456789_".contains(c))(input)?;
    let number = chars.parse().unwrap();
    let value = ValuePtr::new(Value::IntLiteral(number));
    Ok((input, value))
}

fn parse_float_literal(input: &str) -> Result<ValuePtr> {
    // TODO: Error.
    // TODO: Make this better.
    let (input, chars) = take_while1(|c| "0123456789_.e+-".contains(c))(input)?;
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
            let (input, text) = parse_identifier_text(input)?;
            if let Some(local) = scope.all_locals.get(text) {
                Ok((input, local.ptr_clone()))
            } else {
                fail(input)
            }
        }
    }
}

fn parse_assignment_lhs<'b>(
    scope: &'b mut Scope,
    declaration_mode: bool,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, LocalPtr> + 'b {
    move |input| {
        let (input, label) = alt((
            tag("local"),
            tag("input"),
            tag("output"),
            tag("ct_local"),
            tag("ct_input"),
            tag("ct_output"),
            tag(""),
        ))(input)?;
        if label == "" && declaration_mode {
            return fail(input);
        }
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
        let local = if label == "" || declaration_mode {
            if let Some(local) = scope.all_locals.get(name) {
                local.ptr_clone()
            } else {
                return fail(input);
            }
        } else {
            LocalPtr::new(Local {
                compile_time_only: label.contains("ct"),
                name: name.to_owned(),
                typee,
            })
        };
        if !declaration_mode {
            scope.all_locals.insert(name.to_owned(), local.ptr_clone());
            if label.contains("local") {
                scope.plain_locals.push(local.ptr_clone());
            } else if label.contains("input") {
                scope.inputs.push(local.ptr_clone());
            } else if label.contains("output") {
                scope.outputs.push(local.ptr_clone());
            }
        }
        Ok((input, local))
    }
}

fn parse_assignment_statement<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, ValuePtr> + 'b {
    move |input| {
        let mut targets = Vec::new();
        let mut input = input;
        loop {
            let (new_input, target) = parse_assignment_lhs(scope, false)(input)?;
            targets.push(target);
            let (new_input, had_comma) = opt(tuple((ws, tag(","), ws)))(new_input)?;
            let (new_input, had_equals) = opt(tuple((ws, tag("="), ws)))(new_input)?;
            input = new_input;
            if had_equals.is_some() {
                break;
            }
            if had_comma.is_none() {
                return fail(new_input);
            }
        }
        if targets.len() == 0 {
            return fail(input);
        }
        let (input, base) = parse_basic_expression(scope)(input)?;
        let value = ValuePtr::new(Value::Assignment { base, targets });
        Ok((input, value))
    }
}

fn parse_declaration_statement<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, ()> + 'b {
    move |input| {
        let mut targets = Vec::new();
        let mut input = input;
        loop {
            let (new_input, target) = opt(parse_assignment_lhs(scope, true))(input)?;
            if let Some(target) = target {
                targets.push(target);
            } else {
                break;
            }
            let (new_input, had_comma) = opt(tuple((ws, tag(","), ws)))(new_input)?;
            input = new_input;
            if had_comma.is_none() {
                break;
            }
        }
        if targets.len() == 0 {
            return fail(input);
        }
        Ok((input, ()))
    }
}

fn parse_statement<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, Option<ValuePtr>> + 'b {
    move |input| {
        {
            let result = opt(parse_assignment_statement(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, Some(result)));
            }
        }
        {
            let result = opt(parse_declaration_statement(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, None));
            }
        }
        {
            let result = opt(parse_basic_expression(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, Some(result)));
            }
        }
        fail(input)
    }
}

fn parse_basic_expression<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, ValuePtr> + 'b {
    move |input| {
        {
            let result = opt(parse_expression_5(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        fail(input)
    }
}

fn parse_expression_5<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, ValuePtr> + 'b {
    move |input| {
        {
            let result = opt(parse_expression_4(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        fail(input)
    }
}

fn parse_expression_4<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, ValuePtr> + 'b {
    move |input| {
        {
            let result = opt(parse_expression_3(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        fail(input)
    }
}

fn parse_expression_3<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, ValuePtr> + 'b {
    move |input| {
        {
            let result = opt(parse_expression_2(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        fail(input)
    }
}

fn parse_expression_2<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, ValuePtr> + 'b {
    move |input| {
        {
            let result = opt(parse_expression_1(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        fail(input)
    }
}

fn parse_expression_1<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, ValuePtr> + 'b {
    move |input| {
        {
            let result = opt(parse_expression_0(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        fail(input)
    }
}

fn parse_expression_0<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, ValuePtr> + 'b {
    move |input| {
        {
            let result = opt(parse_int_literal)(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        {
            let result = opt(parse_float_literal)(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        {
            let result = opt(parse_bool_literal)(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        {
            let result = opt(parse_identifier_into_value(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        fail(input)
    }
}
