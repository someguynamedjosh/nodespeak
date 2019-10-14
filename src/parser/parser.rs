extern crate pest;

use crate::problem;
use crate::problem::CompileProblem;
use crate::problem::FilePosition;

use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct WaveguideParser;

pub type ParseResult<'a> = Pairs<'a, Rule>;
pub type ParseError = Error<Rule>;

pub use Rule as AstRule;

fn rule_name(rule: &Rule) -> &'static str {
    match rule {
        Rule::WHITESPACE => "whitespace",
        Rule::EOI => "end of file",

        Rule::dec_int => "integer literal",
        Rule::hex_int => "hexadecimal literal",
        Rule::oct_int => "octal literal",
        Rule::legacy_oct_int => "c-style octal literal",
        Rule::bin_int => "binary literal",
        Rule::dec_digit => "digit",
        Rule::float => "float literal",
        Rule::int => "int literal",
        Rule::array_literal => "array literal",
        Rule::literal => "literal value",
        Rule::identifier => "identifier",

        Rule::expr_part_1 => "expression",
        Rule::expr_part_2 => "expression",
        Rule::expr_part => "expression",
        Rule::expr => "expression",
        Rule::negate => "unary negation",
        Rule::index_expr => "array access",
        Rule::operator => "binary operator",

        Rule::func_expr_input_list => "input list for function call",
        Rule::inline_output => "inline keyword",
        Rule::inline_var_dec => "inline variable declaration",
        Rule::func_expr_output_list => "output list for function call",
        Rule::func_expr => "single-output function expression",

        Rule::named_data_type => "name of a data type",
        Rule::dynamic_data_type => "dynamic data type expression",
        Rule::basic_data_type => "data type",
        Rule::array_data_type => "array data type",
        Rule::data_type => "data type",

        Rule::named_function_parameter => "function parameter definition",
        Rule::function_inputs => "input list for function definition",
        Rule::function_outputs => "output list for function definition",
        Rule::single_function_output => "single output for function definition",
        Rule::function_signature => "signature for function definition",
        Rule::function_definition => "function definition",

        Rule::empty_variable => "uninitialized variable name",
        Rule::assigned_variable => "initialized variable declaration",
        Rule::create_variable => "variable declaration",
        Rule::create_variable_statement => "variable declaration statement",
        Rule::input_variable_statement => "input declaration statement",
        Rule::output_variable_statement => "output declaration statement",

        Rule::assign_array_access => "LHS assignment indexing",
        Rule::assign_expr => "LHS assignment expression",
        Rule::assign_statement => "assignment expression",

        Rule::code_block => "code block",
        Rule::returnable_code_block => "code block",
        Rule::return_statement => "return statement",

        Rule::assert_statement => "assert statement",

        Rule::raw_expr_statement => "expression as statement",
        Rule::statement => "statement",
        Rule::root => "program",
    }
}

pub fn parse(text: &str) -> Result<ParseResult, CompileProblem> {
    WaveguideParser::parse(Rule::root, text).map_err(|parse_err| {
        problem::bad_syntax(
            FilePosition::from_input_location(parse_err.location),
            match parse_err.variant {
                ErrorVariant::ParsingError {
                    positives,
                    negatives,
                } => format!(
                    "Expected {}... but found {}.",
                    {
                        positives
                            .iter()
                            .map(|rule| rule_name(rule))
                            .collect::<Vec<&str>>()
                            .join(", ")
                    },
                    {
                        if negatives.len() == 0 {
                            "unknown syntax".to_owned()
                        } else {
                            negatives
                                .iter()
                                .map(|rule| rule_name(rule))
                                .collect::<Vec<&str>>()
                                .join(", ")
                        }
                    }
                ),
                ErrorVariant::CustomError { message: _message } => {
                    unreachable!("Only parsing errors are encountered in the parser.")
                }
            },
        )
    })
}

// We have to put this here because pest does not allow us to export the auto
// generated Rule enum.
pub mod convert {
    use super::*;
    use crate::structure::*;

    fn parse_float(input: &str) -> f64 {
        input
            .replace("_", "")
            .parse()
            .expect("Grammar requires valid float.")
    }

    fn parse_dec_int(input: &str) -> i64 {
        input
            .replace("_", "")
            .parse()
            .expect("Grammar requires valid int.")
    }

    fn parse_hex_int(input: &str) -> i64 {
        // Slice trims off 0x at beginning.
        i64::from_str_radix(&input.replace("_", "")[2..], 16)
            .expect("Grammar requires valid hexadecimal int.")
    }

    fn parse_oct_int(input: &str) -> i64 {
        // Slice trims off 0o at beginning.
        i64::from_str_radix(&input.replace("_", "")[2..], 8)
            .expect("Grammar requires valid octal int.")
    }

    fn parse_legacy_oct_int(input: &str) -> i64 {
        // Slice trims off 0 at beginning.
        i64::from_str_radix(&input.replace("_", "")[1..], 8)
            .expect("Grammar requires valid octal int.")
    }

    fn parse_bin_int(input: &str) -> i64 {
        // Slice trims off 0b at beginning.
        i64::from_str_radix(&input.replace("_", "")[2..], 2)
            .expect("Grammar requires valid binary int.")
    }

    fn lookup_symbol_with_error(
        program: &Program,
        scope: ScopeId,
        symbol: &Pair<Rule>,
    ) -> Result<VariableId, CompileProblem> {
        match program.lookup_symbol(scope, symbol.as_str()) {
            Option::Some(entity) => Result::Ok(entity),
            Option::None => Result::Err(problem::no_entity_with_name(FilePosition::from_pair(
                symbol,
            ))),
        }
    }

    fn convert_func_expr_input_list(
        program: &mut Program,
        scope: ScopeId,
        func_var: VariableId,
        input: Pair<Rule>,
    ) -> Result<Vec<Expression>, CompileProblem> {
        let mut setup = Vec::new();
        for (index, child) in input.into_inner().enumerate() {
            debug_assert!(child.as_rule() == Rule::expr, "Required by grammar.");
            let child_pos = FilePosition::from_pair(&child);
            setup.push(Expression::Assign {
                target: Box::new(Expression::PickInput(func_var, index, child_pos.clone())),
                value: Box::new(convert_expression(program, scope, true, child)?),
                position: child_pos
            });
        }
        Result::Ok(setup)
    }

    fn convert_func_expr(
        program: &mut Program,
        scope: ScopeId,
        force_func_output: bool,
        input: Pair<Rule>,
    ) -> Result<Expression, CompileProblem> {
        // TODO: Handle multi output and no input function calls.
        let input_pos = FilePosition::from_pair(&input);

        let mut input_iter = input.into_inner();
        let function_identifier = input_iter.next().expect("Required by grammar.");
        let identifier_pos = FilePosition::from_pair(&function_identifier);
        let input_list = input_iter.next().expect("Required by grammar.");

        let function_var = lookup_symbol_with_error(program, scope, &function_identifier)?;
        let setup = convert_func_expr_input_list(program, scope, function_var, input_list)?;
        Result::Ok(Expression::FuncCall {
            function: Box::new(Expression::Variable(function_var, identifier_pos)),
            setup,
            teardown: if force_func_output {
                vec![Expression::InlineReturn(
                    Box::new(Expression::PickOutput(function_var, 0, input_pos.clone())),
                    input_pos.clone()
                )]
            } else {
                vec![]
            },
            position: input_pos,
        })
    }

    fn convert_array_literal(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<Expression, CompileProblem> {
        let input_pos = FilePosition::from_pair(&input);
        let mut items = Vec::new();
        for child in input.into_inner() {
            items.push(convert_expression(program, scope, true, child)?);
        }
        return Result::Ok(Expression::Collect(items, input_pos));
    }

    fn convert_expr_part_1(
        program: &mut Program,
        scope: ScopeId,
        force_func_output: bool,
        input: Pair<Rule>,
    ) -> Result<Expression, CompileProblem> {
        for child in input.into_inner() {
            let child_pos = FilePosition::from_pair(&child);
            match child.as_rule() {
                Rule::bin_int => {
                    let value = parse_bin_int(child.as_str());
                    return Result::Ok(Expression::Literal(KnownData::Int(value), child_pos));
                }
                Rule::hex_int => {
                    let value = parse_hex_int(child.as_str());
                    return Result::Ok(Expression::Literal(KnownData::Int(value), child_pos));
                }
                Rule::oct_int => {
                    let value = parse_oct_int(child.as_str());
                    return Result::Ok(Expression::Literal(KnownData::Int(value), child_pos));
                }
                Rule::legacy_oct_int => {
                    // TODO: Warning for using legacy oct format.
                    let value = parse_legacy_oct_int(child.as_str());
                    return Result::Ok(Expression::Literal(KnownData::Int(value), child_pos));
                }
                Rule::dec_int => {
                    let value = parse_dec_int(child.as_str());
                    return Result::Ok(Expression::Literal(KnownData::Int(value), child_pos));
                }
                Rule::float => {
                    let value = parse_float(child.as_str());
                    return Result::Ok(Expression::Literal(KnownData::Float(value), child_pos));
                }
                Rule::func_expr => {
                    return convert_func_expr(program, scope, force_func_output, child)
                }
                Rule::identifier => {
                    return Result::Ok(Expression::Variable(
                        lookup_symbol_with_error(&program, scope, &child)?,
                        child_pos,
                    ))
                }
                Rule::expr => {
                    return convert_expression(program, scope, true, child);
                }
                Rule::array_literal => {
                    return convert_array_literal(program, scope, child);
                }
                _ => unreachable!(),
            }
        }
        unreachable!();
    }

    fn convert_negate(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<Expression, CompileProblem> {
        let input_pos = FilePosition::from_pair(&input);
        let mut input_iter = input.into_inner();
        let child_pair = input_iter.next().expect("Required by grammar.");
        let child_expr = match child_pair.as_rule() {
            Rule::expr_part_1 => convert_expr_part_1(program, scope, true, child_pair)?,
            Rule::index_expr => convert_index_expr(program, scope, child_pair)?,
            _ => unreachable!("Grammar specifies no other children."),
        };
        Result::Ok(Expression::UnaryOperation(
            UnaryOperator::Negate,
            Box::new(child_expr),
            input_pos,
        ))
    }

    fn convert_index_expr(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<Expression, CompileProblem> {
        let input_pos = FilePosition::from_pair(&input);
        let mut iter = input.into_inner();
        let base = convert_expr_part_1(
            program,
            scope,
            true,
            iter.next().expect("Required by grammer."),
        )?;
        let mut indexes = Vec::new();
        for child in iter {
            match child.as_rule() {
                Rule::expr_part_1 => unreachable!("Already dealt with above."),
                Rule::expr => indexes.push(convert_expression(program, scope, true, child)?),
                _ => unreachable!("Grammar specifies no other children."),
            }
        }
        Result::Ok(match base {
            Expression::Access {
                base,
                indexes: mut existing_indexes,
                ..
            } => {
                existing_indexes.append(&mut indexes);
                Expression::Access {
                    base,
                    indexes: existing_indexes,
                    position: input_pos,
                }
            }
            _ => Expression::Access {
                base: Box::new(base),
                indexes,
                position: input_pos,
            },
        })
    }

    fn convert_expr_part(
        program: &mut Program,
        scope: ScopeId,
        force_func_output: bool,
        input: Pair<Rule>,
    ) -> Result<Expression, CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr_part_1 => {
                    return convert_expr_part_1(program, scope, force_func_output, child);
                }
                Rule::index_expr => {
                    return convert_index_expr(program, scope, child);
                }
                Rule::negate => {
                    return convert_negate(program, scope, child);
                }
                _ => unreachable!(),
            }
        }
        unreachable!();
    }

    #[derive(Clone)]
    struct Operator {
        pub id: u32,
        pub precedence: u32,
        pub left_assoc: bool,
    }

    const SENTINEL: Operator = Operator {
        id: 00,
        precedence: 0,
        left_assoc: true,
    };
    const POWER: Operator = Operator {
        id: 07,
        precedence: 19,
        left_assoc: false,
    };
    const MULTIPLY: Operator = Operator {
        id: 03,
        precedence: 18,
        left_assoc: true,
    };
    const DIVIDE: Operator = Operator {
        id: 04,
        precedence: 18,
        left_assoc: true,
    };
    const INT_DIV: Operator = Operator {
        id: 05,
        precedence: 18,
        left_assoc: true,
    };
    const MODULO: Operator = Operator {
        id: 06,
        precedence: 18,
        left_assoc: true,
    };
    const ADD: Operator = Operator {
        id: 01,
        precedence: 17,
        left_assoc: true,
    };
    const SUBTRACT: Operator = Operator {
        id: 02,
        precedence: 17,
        left_assoc: true,
    };
    const LTE: Operator = Operator {
        id: 08,
        precedence: 16,
        left_assoc: true,
    };
    const LT: Operator = Operator {
        id: 09,
        precedence: 16,
        left_assoc: true,
    };
    const GTE: Operator = Operator {
        id: 10,
        precedence: 16,
        left_assoc: true,
    };
    const GT: Operator = Operator {
        id: 11,
        precedence: 16,
        left_assoc: true,
    };
    const EQ: Operator = Operator {
        id: 12,
        precedence: 15,
        left_assoc: true,
    };
    const NEQ: Operator = Operator {
        id: 13,
        precedence: 15,
        left_assoc: true,
    };
    const BAND: Operator = Operator {
        id: 14,
        precedence: 14,
        left_assoc: true,
    };
    const BXOR: Operator = Operator {
        id: 15,
        precedence: 13,
        left_assoc: true,
    };
    const BOR: Operator = Operator {
        id: 16,
        precedence: 12,
        left_assoc: true,
    };
    const AND: Operator = Operator {
        id: 17,
        precedence: 11,
        left_assoc: true,
    };
    const XOR: Operator = Operator {
        id: 18,
        precedence: 10,
        left_assoc: true,
    };
    const OR: Operator = Operator {
        id: 19,
        precedence: 9,
        left_assoc: true,
    };

    fn op_str_to_operator(op_str: &str) -> Operator {
        if op_str == "**" {
            POWER
        } else if op_str == "+" {
            ADD
        } else if op_str == "-" {
            SUBTRACT
        } else if op_str == "*" {
            MULTIPLY
        } else if op_str == "/" {
            DIVIDE
        } else if op_str == "//" {
            INT_DIV
        } else if op_str == "%" {
            MODULO
        } else if op_str == "<=" {
            LTE
        } else if op_str == "<" {
            LT
        } else if op_str == ">=" {
            GTE
        } else if op_str == ">" {
            GT
        } else if op_str == "==" {
            EQ
        } else if op_str == "!=" {
            NEQ
        } else if op_str == "band" {
            BAND
        } else if op_str == "bxor" {
            BXOR
        } else if op_str == "bor" {
            BOR
        } else if op_str == "and" {
            AND
        } else if op_str == "xor" {
            XOR
        } else if op_str == "or" {
            OR
        } else {
            unreachable!();
        }
    }

    fn apply_operator(
        operator: &Operator,
        operand_1: Expression,
        operand_2: Expression,
        position: FilePosition,
    ) -> Expression {
        let operand_1 = Box::new(operand_1);
        let operand_2 = Box::new(operand_2);
        let operator = if operator.id == ADD.id {
            BinaryOperator::Add
        } else if operator.id == SUBTRACT.id {
            BinaryOperator::Subtract
        } else if operator.id == MULTIPLY.id {
            BinaryOperator::Multiply
        } else if operator.id == DIVIDE.id {
            BinaryOperator::Divide
        } else if operator.id == INT_DIV.id {
            BinaryOperator::IntDiv
        } else if operator.id == MODULO.id {
            BinaryOperator::Modulo
        } else if operator.id == POWER.id {
            BinaryOperator::Power
        } else if operator.id == LTE.id {
            BinaryOperator::LessThanOrEqual
        } else if operator.id == LT.id {
            BinaryOperator::LessThan
        } else if operator.id == GTE.id {
            BinaryOperator::GreaterThanOrEqual
        } else if operator.id == GT.id {
            BinaryOperator::GreaterThan
        } else if operator.id == EQ.id {
            BinaryOperator::Equal
        } else if operator.id == NEQ.id {
            BinaryOperator::NotEqual
        } else if operator.id == BAND.id {
            BinaryOperator::BAnd
        } else if operator.id == BXOR.id {
            BinaryOperator::BXor
        } else if operator.id == BOR.id {
            BinaryOperator::BOr
        } else if operator.id == AND.id {
            BinaryOperator::And
        } else if operator.id == XOR.id {
            BinaryOperator::Xor
        } else if operator.id == OR.id {
            BinaryOperator::Or
        } else {
            unreachable!();
        };
        Expression::BinaryOperation(operand_1, operator, operand_2, position)
    }

    fn convert_expression(
        program: &mut Program,
        scope: ScopeId,
        force_func_output: bool,
        input: Pair<Rule>,
    ) -> Result<Expression, CompileProblem> {
        let mut operand_stack = Vec::with_capacity(64);
        let mut operator_stack = Vec::with_capacity(64);
        operator_stack.push(SENTINEL);

        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr_part => {
                    let result = convert_expr_part(program, scope, force_func_output, child)?;
                    operand_stack.push(result);
                }
                Rule::operator => {
                    let op_str = child.as_str();
                    let new_operator = op_str_to_operator(op_str);
                    // Shunting yard algorithm.
                    // TODO: Implement right-associative operators.
                    loop {
                        let top_op_prec = operator_stack.last().cloned().unwrap().precedence;
                        if new_operator.precedence >= top_op_prec {
                            operator_stack.push(new_operator);
                            break;
                        } else {
                            let top_operator = operator_stack.pop().unwrap();
                            // The stack is in reverse order, pop the RHS first.
                            let operand_2 = operand_stack.pop().unwrap();
                            let operand_1 = operand_stack.pop().unwrap();
                            let mut position = operand_1.clone_position();
                            position.include_other(&operand_2.clone_position());
                            operand_stack.push(apply_operator(
                                &top_operator,
                                operand_1,
                                operand_2,
                                position,
                            ));
                        }
                    }
                }
                _ => unreachable!("Grammar specifies no other children."),
            }
        }

        // If true, we dealt with all the operators in the loop, so just return the only 'operand'
        // on the stack.
        if operator_stack.len() == 1 {
            return Result::Ok(operand_stack.pop().unwrap());
        }

        // Otherwise, we need to do some more looping to get rid of all the extra operators. The
        // shunting yard algorithm used above guarantees that we can just loop through and compose
        // them in order because they are already in the correct order of precedence.
        loop {
            let top_operator = operator_stack.pop().unwrap();
            let operand_2 = operand_stack.pop().unwrap();
            let operand_1 = operand_stack.pop().unwrap();
            let mut position = operand_1.clone_position();
            position.include_other(&operand_2.clone_position());
            let result = apply_operator(&top_operator, operand_1, operand_2, position);
            // The last operator is the sentinel, we want to exit before we reach it.
            if operator_stack.len() == 1 {
                return Result::Ok(result);
            } else {
                operand_stack.push(result);
            }
        }
    }

    fn convert_assign_expr(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<Expression, CompileProblem> {
        let input_pos = FilePosition::from_pair(&input);
        let mut input_iter = input.into_inner();
        let child = input_iter.next().expect("Identifier required by grammar.");
        let identifier_pos = FilePosition::from_pair(&child);
        let base_var = lookup_symbol_with_error(&program, scope, &child)?;
        let mut indexes = Vec::new();
        for child in input_iter {
            match child.as_rule() {
                Rule::identifier => unreachable!("Already handled above."),
                Rule::assign_array_access => indexes.push(convert_expression(
                    program,
                    scope,
                    true,
                    child.into_inner().next().expect("Required by grammar."),
                )?),
                _ => unreachable!("Grammar specifies no other children."),
            }
        }
        Result::Ok(if indexes.len() > 0 {
            Expression::Access {
                base: Box::new(Expression::Variable(base_var, identifier_pos)),
                indexes,
                position: input_pos,
            }
        } else {
            Expression::Variable(base_var, input_pos)
        })
    }

    // Creates a variable, returns its id.
    fn parse_named_function_parameter(
        program: &mut Program,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<VariableId, CompileProblem> {
        let mut name = Option::None;
        let mut data_type = Option::None;
        let input_pos = FilePosition::from_pair(&input);
        for part in input.into_inner() {
            match part.as_rule() {
                Rule::data_type => {
                    data_type = Option::Some(convert_data_type(program, func_scope, part)?)
                }
                Rule::identifier => name = Option::Some(part.as_str()),
                _ => unreachable!(),
            }
        }
        let variable = Variable::variable(input_pos, data_type.unwrap(), None);
        Result::Ok(program.adopt_and_define_symbol(func_scope, name.unwrap(), variable))
    }

    fn add_function_inputs(
        program: &mut Program,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        for child in input.into_inner() {
            let new_input = parse_named_function_parameter(program, func_scope, child)?;
            program[func_scope].add_input(new_input);
        }
        Result::Ok(())
    }

    fn add_function_outputs(
        program: &mut Program,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        for child in input.into_inner() {
            let new_output = parse_named_function_parameter(program, func_scope, child)?;
            program[func_scope].add_output(new_output);
        }
        Result::Ok(())
    }

    fn add_function_output(
        program: &mut Program,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let mut data_type = Option::None;
        let input_pos = FilePosition::from_pair(&input);
        for part in input.into_inner() {
            match part.as_rule() {
                Rule::data_type => {
                    data_type = Option::Some(convert_data_type(program, func_scope, part)?)
                }
                _ => unreachable!(),
            }
        }
        let variable = Variable::variable(input_pos, data_type.unwrap(), None);
        let new_output = program.adopt_and_define_symbol(func_scope, "!return_value", variable);
        program[func_scope].add_output(new_output);
        Result::Ok(())
    }

    fn convert_function_signature(
        program: &mut Program,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::function_inputs => add_function_inputs(program, func_scope, child)?,
                Rule::function_outputs => add_function_outputs(program, func_scope, child)?,
                Rule::single_function_output => add_function_output(program, func_scope, child)?,
                _ => unreachable!(),
            }
        }
        Result::Ok(())
    }

    fn convert_returnable_code_block(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<Option<Expression>, CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::statement => convert_statement(program, scope, child)?,
                Rule::expr => {
                    return Result::Ok(Option::Some(convert_expression(
                        program, scope, true, child,
                    )?));
                }
                _ => unreachable!(),
            }
        }
        Result::Ok(None)
    }

    fn convert_function_definition(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let mut real_header_position = FilePosition::start_at(&input);
        let mut input_iter = input.into_inner();
        let name = {
            let child = input_iter.next().expect("Required by grammar.");
            real_header_position.include(&child);
            child.as_str()
        };
        let func_scope = program.create_child_scope(scope);
        let mut function = FunctionData::new(func_scope, FilePosition::placeholder());
        for child in input_iter {
            match child.as_rule() {
                Rule::identifier => unreachable!("Handled above"),
                Rule::function_signature => {
                    real_header_position.include(&child);
                    convert_function_signature(program, func_scope, child)?;
                }
                Rule::returnable_code_block => {
                    function.set_header(real_header_position);
                    // So that code inside the body can refer to the function.
                    program.adopt_and_define_symbol(scope, name, Variable::function_def(function));

                    let possible_output =
                        convert_returnable_code_block(program, func_scope, child)?;
                    if let Option::Some(output) = possible_output {
                        let output_pos = output.clone_position();
                        if let Option::Some(output_var) = program[func_scope].get_single_output() {
                            program[func_scope].add_expression(Expression::Assign {
                                target: Box::new(Expression::Variable(
                                    output_var,
                                    output_pos.clone(),
                                )),
                                value: Box::new(output),
                                position: output_pos,
                            });
                        }
                    }
                    // This branch arm can only be expressioned once but I don't know how to tell rustc that,
                    // so we use a break statement for that purpose. Since the code block is the last element
                    // parsed anyway, it doesn't change how the code works.
                    break;
                }
                _ => unreachable!("Grammar specifies no other children."),
            }
        }
        Result::Ok(())
    }

    fn convert_assigned_variable(
        program: &mut Program,
        scope: ScopeId,
        data_type: DataType,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let input_pos = FilePosition::from_pair(&input);
        let mut input_iter = input.into_inner();
        let (name, variable_position) = {
            let child = input_iter.next().expect("Required by grammar.");
            let position = FilePosition::from_pair(&child);
            (child.as_str(), position)
        };
        let variable_id = {
            let variable = Variable::variable(input_pos.clone(), data_type, None);
            program.adopt_and_define_symbol(scope, name, variable)
        };
        let expr_pair = input_iter.next().expect("Required by grammar.");
        let expr = convert_expression(program, scope, true, expr_pair).map_err(|mut err| {
            problem::hint_encountered_while_parsing(
                "initial value for a variable",
                input_pos.clone(),
                &mut err,
            );
            err
        })?;
        program[scope].add_expression(Expression::CreationPoint(variable_id, input_pos.clone()));
        program[scope].add_expression(Expression::Assign {
            target: Box::new(Expression::Variable(variable_id, variable_position.clone())),
            value: Box::new(expr),
            position: input_pos.clone(),
        });
        Result::Ok(())
    }

    fn convert_empty_variable(
        program: &mut Program,
        scope: ScopeId,
        data_type: DataType,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let input_pos = FilePosition::from_pair(&input);
        let name = input
            .into_inner()
            .next()
            .expect("Required by grammar.")
            .as_str();
        let variable = Variable::variable(input_pos.clone(), data_type, None);
        let variable_id = program.adopt_and_define_symbol(scope, name, variable);
        program[scope].add_expression(Expression::CreationPoint(variable_id, input_pos));
        Result::Ok(())
    }

    fn convert_basic_data_type(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<DataType, CompileProblem> {
        let type_variable_id;
        let child = input
            .into_inner()
            .next()
            .expect("Grammar requires one child.");
        match child.as_rule() {
            Rule::named_data_type => {
                let sub_child = child
                    .into_inner()
                    .next()
                    .expect("Grammar requires one child.");
                match sub_child.as_rule() {
                    Rule::identifier => {
                        type_variable_id = lookup_symbol_with_error(&program, scope, &sub_child)?;
                    }
                    _ => unreachable!(),
                }
            }
            Rule::dynamic_data_type => unimplemented!(),
            _ => unreachable!(),
        }
        let type_variable = &program[type_variable_id];
        Result::Ok(match type_variable.borrow_initial_value() {
            KnownData::DataType(real_type) => real_type.clone(),
            _ => BaseType::Dynamic(type_variable_id).to_scalar_type(),
        })
    }

    fn convert_array_data_type(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<DataType, CompileProblem> {
        // We have to store the dimensions and process them later because the base type comes after
        // all the dimensions.
        let mut dimensions = Vec::new();
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr => dimensions.push(convert_expression(program, scope, true, child)?),
                Rule::basic_data_type => {
                    // Array data type stores dimensions in the same order as the grammar, biggest to
                    // smallest.
                    let mut data_type = convert_basic_data_type(program, scope, child)?;
                    data_type.wrap_with_dimensions(dimensions);
                    return Result::Ok(data_type);
                }
                _ => unreachable!("Grammar allows no other children."),
            }
        }
        unreachable!("Grammar requires base data type.");
    }

    fn convert_data_type(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<DataType, CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::array_data_type => return convert_array_data_type(program, scope, child),
                Rule::basic_data_type => return convert_basic_data_type(program, scope, child),
                _ => unreachable!(),
            }
        }
        unreachable!();
    }

    fn convert_input_variable_statement(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        if program[scope].get_parent().is_some() {
            return Result::Err(problem::io_inside_function(FilePosition::from_pair(&input)));
        }
        let mut input_iter = input.into_inner();
        let data_type = {
            let child = input_iter.next().expect("Required by grammar.");
            convert_data_type(program, scope, child)?
        };
        for child in input_iter {
            if let Rule::identifier = child.as_rule() {
                let position = FilePosition::from_pair(&child);
                let variable = Variable::variable(position.clone(), data_type.clone(), None);
                let var_id = program.adopt_and_define_symbol(scope, child.as_str(), variable);
                program[scope].add_input(var_id);
                program[scope].add_expression(Expression::CreationPoint(var_id, position.clone()));
            } else if let Rule::data_type = child.as_rule() {
                unreachable!("Handled above.");
            } else {
                unreachable!("Grammar specifies no other children.");
            }
        }
        Result::Ok(())
    }

    fn convert_output_variable_statement(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        if program[scope].get_parent().is_some() {
            return Result::Err(problem::io_inside_function(FilePosition::from_pair(&input)));
        }
        let mut input_iter = input.into_inner();
        let data_type = {
            let child = input_iter.next().expect("Required by grammar.");
            convert_data_type(program, scope, child)?
        };
        for child in input_iter {
            if let Rule::identifier = child.as_rule() {
                let position = FilePosition::from_pair(&child);
                let variable = Variable::variable(position.clone(), data_type.clone(), None);
                let var_id = program.adopt_and_define_symbol(scope, child.as_str(), variable);
                program[scope].add_output(var_id);
                program[scope].add_expression(Expression::CreationPoint(var_id, position.clone()));
            } else if let Rule::data_type = child.as_rule() {
                unreachable!("Handled above.");
            } else {
                unreachable!("Grammar specifies no other children.");
            }
        }
        Result::Ok(())
    }

    fn convert_create_variable_statement(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let mut input_iter = input.into_inner();
        let data_type = convert_data_type(
            program,
            scope,
            input_iter.next().expect("Required by grammar."),
        )?;
        for child in input_iter {
            match child.as_rule() {
                Rule::data_type => unreachable!("Handled above."),
                Rule::assigned_variable => {
                    convert_assigned_variable(program, scope, data_type.clone(), child)?
                }
                Rule::empty_variable => {
                    convert_empty_variable(program, scope, data_type.clone(), child)?
                }
                _ => unreachable!(),
            }
        }
        Result::Ok(())
    }

    fn convert_return_statement(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let input_pos = FilePosition::from_pair(&input);
        let mut index = 0;
        let func = program
            .lookup_and_clone_parent_function(scope)
            .ok_or_else(|| problem::return_from_root(FilePosition::from_pair(&input)))?;
        let outputs = program[func.get_body()].borrow_outputs().clone();
        // In case we need to make an error, we can't borrow input once we enter the loop because
        // the loop consumes it.
        let statement_position = FilePosition::from_pair(&input);
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr => {
                    let child_pos = FilePosition::from_pair(&child);
                    if index >= outputs.len() {
                        return Result::Err(problem::extra_return_value(
                            statement_position,
                            FilePosition::from_pair(&child),
                            func.get_header().clone(),
                            outputs.len(),
                        ));
                    }
                    let value =
                        convert_expression(program, scope, true, child).map_err(|mut err| {
                            problem::hint_encountered_while_parsing(
                                "a return statement",
                                statement_position.clone(),
                                &mut err,
                            );
                            err
                        })?;
                    program[scope].add_expression(Expression::Assign {
                        target: Box::new(Expression::Variable(
                            outputs[index],
                            FilePosition::placeholder(),
                        )),
                        value: Box::new(value),
                        position: child_pos,
                    });
                    index += 1;
                }
                _ => unreachable!(),
            }
        }
        if index != 0 && index < outputs.len() {
            return Result::Err(problem::missing_return_values(
                statement_position,
                func.get_header().clone(),
                outputs.len(),
                index,
            ));
        }
        program[scope].add_expression(Expression::Return(input_pos));
        Result::Ok(())
    }

    fn convert_assert_statement(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let input_pos = FilePosition::from_pair(&input);
        let mut input_iter = input.into_inner();
        let value = {
            let value_input = input_iter.next().expect("Required by grammar.");
            convert_expression(program, scope, true, value_input)?
        };
        program[scope].add_expression(Expression::Assert(Box::new(value), input_pos));
        Result::Ok(())
    }

    fn convert_statement(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::function_definition => convert_function_definition(program, scope, child)?,
                Rule::code_block => unimplemented!(),
                Rule::return_statement => convert_return_statement(program, scope, child)?,
                Rule::assert_statement => convert_assert_statement(program, scope, child)?,
                Rule::input_variable_statement => {
                    convert_input_variable_statement(program, scope, child)?
                }
                Rule::output_variable_statement => {
                    convert_output_variable_statement(program, scope, child)?
                }
                Rule::create_variable_statement => {
                    convert_create_variable_statement(program, scope, child)?
                }
                Rule::assign_statement => {
                    let child_pos = FilePosition::from_pair(&child);
                    let mut child_iter = child.into_inner();
                    let output = {
                        let assign_expr = child_iter.next().unwrap();
                        debug_assert!(Rule::assign_expr == assign_expr.as_rule());
                        convert_assign_expr(program, scope, assign_expr)?
                    };
                    let value = {
                        let expr = child_iter.next().unwrap();
                        debug_assert!(Rule::expr == expr.as_rule());
                        convert_expression(program, scope, true, expr)?
                    };
                    program[scope].add_expression(Expression::Assign {
                        target: Box::new(output),
                        value: Box::new(value),
                        position: child_pos,
                    });
                }
                Rule::expr => {
                    let expression = convert_expression(program, scope, false, child)?;
                    program[scope].add_expression(expression);
                }
                _ => unreachable!(),
            }
        }
        Result::Ok(())
    }

    pub fn convert_ast_to_structure(input: &mut ParseResult) -> Result<Program, CompileProblem> {
        let root = input.next().unwrap();
        let mut program = Program::new();
        let scope = program.get_entry_point();

        for statement in root.into_inner() {
            match statement.as_rule() {
                Rule::EOI => continue,
                Rule::statement => convert_statement(&mut program, scope, statement)?,
                _ => unreachable!(),
            }
        }

        Result::Ok(program)
    }
}
