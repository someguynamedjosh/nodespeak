use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    combinator::{fail, opt},
    multi::many0,
    sequence::{terminated, tuple},
    IResult, Parser,
};

use crate::values::{BuiltinOp, BuiltinType, Index, Local, LocalPtr, Value, ValuePtr};

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

pub fn parse_root(input: &str) -> Result<(Scope, Vec<ValuePtr>)> {
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
            (
                scope,
                values.into_iter().flat_map(|x| x.into_iter()).collect(),
            ),
        ))
    }
}

fn parse_body<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, Vec<ValuePtr>> + 'b {
    move |input| {
        let (input, _) = ws(input)?;
        let (input, values) = many0(terminated(
            parse_statement(scope),
            tuple((ws, tag(";"), ws)),
        ))(input)?;
        Ok((
            input,
            values.into_iter().flat_map(|x| x.into_iter()).collect(),
        ))
    }
}

fn ws(input: &str) -> Result<&str> {
    take_while(|c: char| c.is_whitespace())(input)
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
) -> impl for<'a> FnMut(&'a str) -> Result<'a, (LocalPtr, Option<Index>)> + 'b {
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
        let (input, indices) = opt(parse_argument_list(scope))(input)?;
        if indices.is_some() {
            assert_eq!(
                label, "",
                "Cannot simultaneously declare and index a local."
            );
        }
        let (input, _) = ws(input)?;
        let (input, keyword_8wide) = opt(tag("8wide"))(input)?;
        let (input, _) = ws(input)?;
        let (input, has_colon) = opt(tag(":"))(input)?;
        let (input, typee) = if has_colon.is_some() {
            let (input, _) = ws(input)?;
            let (input, typee) = parse_basic_expression(scope)(input)?;
            (input, typee)
        } else {
            (input, ValuePtr::new(Value::BuiltinType(BuiltinType::Any)))
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
        let index = if let Some(indices) = indices {
            Some(Index {
                indices,
                eight_wide_mode: keyword_8wide.is_some(),
            })
        } else {
            None
        };
        Ok((input, (local, index)))
    }
}

fn parse_assignment_statement<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, Vec<ValuePtr>> + 'b {
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
        let value = if targets.len() == 1 {
            let (target, index) = targets.into_iter().next().unwrap();
            vec![ValuePtr::new(Value::Assignment {
                base,
                index,
                target,
            })]
        } else {
            if let Value::FunctionCall(base, args, 0) = &*base.borrow() {
                let mut value = Vec::new();
                for (target_index, (target, index)) in targets.into_iter().enumerate() {
                    value.push(ValuePtr::new(Value::Assignment {
                        base: ValuePtr::new(Value::FunctionCall(
                            base.ptr_clone(),
                            args.clone(),
                            target_index,
                        )),
                        index,
                        target,
                    }));
                }
                value
            } else {
                todo!("Nice error, only function calls can have multiple outputs.")
            }
        };
        Ok((input, value))
    }
}

fn parse_declaration_statement<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, Vec<ValuePtr>> + 'b {
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
        Ok((
            input,
            targets
                .into_iter()
                .map(|x| {
                    debug_assert!(x.1.is_none());
                    ValuePtr::new(Value::Declaration(x.0))
                })
                .collect(),
        ))
    }
}

fn parse_statement<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<'a, Vec<ValuePtr>> + 'b {
    move |input| {
        {
            let result = opt(parse_assignment_statement(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        {
            let result = opt(parse_declaration_statement(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, result));
            }
        }
        {
            let result = opt(parse_basic_expression(scope))(input)?;
            if let (input, Some(result)) = result {
                return Ok((input, vec![result]));
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
            let result = opt(parse_function_def(scope))(input)?;
            if let (input, Some(value)) = result {
                return Ok((input, value));
            }
        }
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
            let result = opt(tag("ANY"))(input)?;
            if let (input, Some(_)) = result {
                return Ok((input, ValuePtr::new(Value::BuiltinType(BuiltinType::Any))));
            }
        }
        {
            let result = opt(tag("MALFORMED"))(input)?;
            if let (input, Some(_)) = result {
                return Ok((input, ValuePtr::new(Value::Malformed)));
            }
        }
        {
            let result = opt(tag("Int"))(input)?;
            if let (input, Some(_)) = result {
                return Ok((input, ValuePtr::new(Value::BuiltinType(BuiltinType::Int))));
            }
        }
        {
            let result = opt(tag("Float"))(input)?;
            if let (input, Some(_)) = result {
                return Ok((input, ValuePtr::new(Value::BuiltinType(BuiltinType::Float))));
            }
        }
        {
            let result = opt(tag("Bool"))(input)?;
            if let (input, Some(_)) = result {
                return Ok((input, ValuePtr::new(Value::BuiltinType(BuiltinType::Bool))));
            }
        }
        {
            let result = opt(tag("Malformed"))(input)?;
            if let (input, Some(_)) = result {
                return Ok((
                    input,
                    ValuePtr::new(Value::BuiltinType(BuiltinType::Malformed)),
                ));
            }
        }
        {
            let result = opt(parse_function_call(scope))(input)?;
            if let (input, Some((name, args))) = result {
                let base = match name {
                    "add" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Add)),
                    "sub" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Sub)),
                    "mul" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Mul)),
                    "div" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Div)),
                    "rem" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Rem)),

                    "gt" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Gt)),
                    "lt" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Lt)),
                    "gte" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Gte)),
                    "lte" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Lte)),
                    "eq" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Eq)),
                    "neq" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Neq)),

                    "and" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::And)),
                    "or" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Or)),
                    "xor" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Xor)),
                    "not" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Not)),

                    "cast" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Cast)),
                    "typeof" => ValuePtr::new(Value::BuiltinOp(BuiltinOp::Typeof)),

                    "Array" => {
                        assert!(args.len() >= 2);
                        let mut args = args.into_iter();
                        let eltype = args.next().unwrap();
                        let dims = args.collect();
                        let value = Value::BuiltinType(BuiltinType::Array { eltype, dims });
                        return Ok((input, ValuePtr::new(value)));
                    }
                    "InSet" => {
                        assert!(args.len() >= 1);
                        let mut iter = args.clone().into_iter();
                        let mut eltype = ValuePtr::new(iter.next().unwrap().typee());
                        for arg in iter {
                            eltype = ValuePtr::new(Value::FunctionCall(
                                ValuePtr::new(Value::BuiltinOp(BuiltinOp::Add)),
                                vec![eltype, ValuePtr::new(arg.typee())],
                                0,
                            ));
                        }
                        let elements = args;
                        let value = Value::BuiltinType(BuiltinType::InSet { eltype, elements });
                        return Ok((input, ValuePtr::new(value)));
                    }
                    "Fn" => todo!(),
                    _ => {
                        if let Some(base) = scope.all_locals.get(name) {
                            ValuePtr::new(Value::Local(base.ptr_clone()))
                        } else {
                            return fail(input);
                        }
                    }
                };
                let value = Value::FunctionCall(base, args, 0);
                return Ok((input, ValuePtr::new(value)));
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

fn parse_comma_expression_list<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<Vec<ValuePtr>> + 'b {
    move |mut input| {
        let mut elements = Vec::new();
        loop {
            let (new_input, element) = parse_basic_expression(scope)(input)?;
            elements.push(element);
            let (new_input, comma) = opt(tuple((ws, tag(","), ws)))(new_input)?;
            input = new_input;
            if comma.is_none() {
                break;
            }
        }
        Ok((input, elements))
    }
}

fn parse_function_call<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<(&'a str, Vec<ValuePtr>)> + 'b {
    move |input| {
        let (input, ident) = parse_identifier_text(input)?;
        let (input, args) = parse_argument_list(scope)(input)?;
        Ok((input, (ident, args)))
    }
}

fn parse_argument_list<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<Vec<ValuePtr>> + 'b {
    move |input| {
        let (input, _) = tuple((ws, tag("("), ws))(input)?;
        let (input, args) = parse_comma_expression_list(scope)(input)?;
        let (input, _) = tuple((ws, tag(")"), ws))(input)?;
        Ok((input, args))
    }
}

fn parse_function_def<'b>(
    scope: &'b mut Scope,
) -> impl for<'a> FnMut(&'a str) -> Result<ValuePtr> + 'b {
    move |input| {
        let mut new_scope = scope.clone();
        new_scope.plain_locals.clear();
        new_scope.inputs.clear();
        new_scope.outputs.clear();
        let (input, _) = tag("fn")(input)?;
        let (input, _) = ws(input)?;
        let (input, _) = tag("{")(input)?;
        let (input, _) = ws(input)?;
        let (input, body) = parse_body(&mut new_scope)(input)?;
        let (input, _) = ws(input)?;
        let (input, _) = tag("}")(input)?;
        Ok((
            input,
            ValuePtr::new(Value::Function {
                inputs: new_scope.inputs,
                outputs: new_scope.outputs,
                locals: new_scope.plain_locals,
                body,
            }),
        ))
    }
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
