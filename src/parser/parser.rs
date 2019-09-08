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

        Rule::func_input_list => "input list for function call",
        Rule::func_output_return_inline => "inline",
        Rule::func_output_var_dec => "declared variable output",
        Rule::func_output_list => "output list for function call",
        Rule::func_output => "output for function call",
        Rule::func_lambda => "lambda definition for function call",
        Rule::lambda_adjective => "adjective for function call",
        Rule::func_expr_extra_list => "zero or more adjectives and lambdas for function call",
        Rule::func_expr => "function call",

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

    fn convert_func_expr_input_list(
        program: &mut Program,
        scope: ScopeId,
        func_call: &mut FuncCall,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr => {
                    let arg_var = convert_expression(program, scope, None, true, child)?;
                    func_call.add_input(arg_var);
                }
                _ => unreachable!(),
            }
        }
        Result::Ok(())
    }

    fn convert_func_expr_output_list(
        program: &mut Program,
        scope: ScopeId,
        func_call: &mut FuncCall,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::func_output_return_inline => unimplemented!(),
                Rule::func_output_var_dec => unimplemented!(),
                Rule::assign_expr => {
                    let into = convert_assign_expr(program, scope, child)?;
                    func_call.add_output(into);
                }
                _ => unreachable!(),
            }
        }
        Result::Ok(())
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

    fn convert_func_expr(
        program: &mut Program,
        scope: ScopeId,
        preferred_output: &Option<VarAccess>,
        force_func_output: bool,
        input: Pair<Rule>,
    ) -> Result<VarAccess, CompileProblem> {
        let mut output_var = Option::None;
        let mut func_call = Option::None;
        let mut explicit_output_list = Option::None;
        let input_pos = FilePosition::from_pair(&input);
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::identifier => {
                    func_call = Option::Some(FuncCall::new(
                        lookup_symbol_with_error(&program, scope, &child)?,
                        input_pos.clone(),
                    ))
                }
                Rule::func_input_list => convert_func_expr_input_list(
                    program,
                    scope,
                    func_call.as_mut().unwrap(),
                    child,
                )?,
                // TODO: Inline return.
                Rule::func_output_list => {
                    explicit_output_list = Option::Some(FilePosition::from_pair(&child));
                    convert_func_expr_output_list(
                        program,
                        scope,
                        func_call.as_mut().unwrap(),
                        child,
                    )?
                }
                Rule::func_lambda => unimplemented!(),
                Rule::lambda_adjective => unimplemented!(),
                _ => unreachable!(),
            }
        }
        if let Option::Some(output_access) = preferred_output {
            // TODO: Inline return.
            if let Option::Some(output_pos) = explicit_output_list {
                return Result::Err(problem::missing_inline_return(input_pos, output_pos));
            }
            match func_call.as_mut() {
                Option::Some(call) => call.add_output(output_access.clone()),
                Option::None => {
                    unreachable!("Should have encountered at least an identifier symbol.")
                }
            }
        } else if force_func_output {
            // TODO: Inline return.
            if let Option::Some(output_pos) = explicit_output_list {
                return Result::Err(problem::missing_inline_return(input_pos, output_pos));
            }
            let temp_var = program.make_intermediate_auto_var(scope, input_pos.clone());
            let output_access = VarAccess::new(input_pos.clone(), temp_var);
            output_var = Option::Some(temp_var);
            match func_call.as_mut() {
                Option::Some(call) => call.add_output(output_access.clone()),
                Option::None => {
                    unreachable!("Should have encountered at least an identifier symbol.")
                }
            }
        }
        program.add_func_call(scope, func_call.unwrap())?;
        if let Option::Some(output_access) = preferred_output {
            Result::Ok(output_access.clone())
        } else {
            Result::Ok(match output_var {
                Option::Some(value) => VarAccess::new(input_pos, value),
                Option::None => VarAccess::new(
                    input_pos.clone(),
                    program.adopt_and_define_intermediate(scope, Variable::void(input_pos.clone())),
                ),
            })
        }
    }

    fn convert_array_literal(
        program: &mut Program,
        scope: ScopeId,
        preferred_output: &Option<VarAccess>,
        input: Pair<Rule>,
    ) -> Result<VarAccess, CompileProblem> {
        let position = FilePosition::from_pair(&input);
        let mut item_pairs = input.into_inner().collect::<Vec<Pair<Rule>>>();
        let size_literal = program.adopt_and_define_intermediate(
            scope,
            Variable::int_literal(position.clone(), item_pairs.len() as i64),
        );
        let data_type = DataType::array(
            BaseType::Automatic,
            vec![VarAccess::new(position.clone(), size_literal)],
        );
        let output_access = match preferred_output {
            Option::Some(output) => output.clone(),
            Option::None => VarAccess::new(
                position.clone(),
                program.adopt_and_define_intermediate(
                    scope,
                    Variable::variable(position.clone(), data_type, None),
                ),
            ),
        };
        for (index, item_pair) in item_pairs.into_iter().enumerate() {
            let index_literal = VarAccess::new(
                FilePosition::from_pair(&item_pair),
                program.adopt_and_define_intermediate(
                    scope,
                    Variable::int_literal(FilePosition::from_pair(&item_pair), index as i64),
                ),
            );
            convert_expression(
                program,
                scope,
                Option::Some(output_access.with_additional_index(index_literal)),
                true,
                item_pair,
            )?;
        }
        return Result::Ok(output_access);
    }

    fn convert_expr_part_1(
        program: &mut Program,
        scope: ScopeId,
        preferred_output: &Option<VarAccess>,
        force_func_output: bool,
        input: Pair<Rule>,
    ) -> Result<VarAccess, CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::bin_int => {
                    let value = parse_bin_int(child.as_str());
                    let var = program.adopt_variable(Variable::int_literal(
                        FilePosition::from_pair(&child),
                        value,
                    ));
                    return Result::Ok(VarAccess::new(FilePosition::from_pair(&child), var));
                }
                Rule::hex_int => {
                    let value = parse_hex_int(child.as_str());
                    let var = program.adopt_variable(Variable::int_literal(
                        FilePosition::from_pair(&child),
                        value,
                    ));
                    return Result::Ok(VarAccess::new(FilePosition::from_pair(&child), var));
                }
                Rule::oct_int => {
                    let value = parse_oct_int(child.as_str());
                    let var = program.adopt_variable(Variable::int_literal(
                        FilePosition::from_pair(&child),
                        value,
                    ));
                    return Result::Ok(VarAccess::new(FilePosition::from_pair(&child), var));
                }
                Rule::legacy_oct_int => {
                    let value = parse_legacy_oct_int(child.as_str());
                    // TODO: Warning for using legacy oct format.
                    let var = program.adopt_variable(Variable::int_literal(
                        FilePosition::from_pair(&child),
                        value,
                    ));
                    return Result::Ok(VarAccess::new(FilePosition::from_pair(&child), var));
                }
                Rule::dec_int => {
                    let value = parse_dec_int(child.as_str());
                    let var = program.adopt_variable(Variable::int_literal(
                        FilePosition::from_pair(&child),
                        value,
                    ));
                    return Result::Ok(VarAccess::new(FilePosition::from_pair(&child), var));
                }
                Rule::float => {
                    let value = parse_float(child.as_str());
                    let var = program.adopt_variable(Variable::float_literal(
                        FilePosition::from_pair(&child),
                        value,
                    ));
                    return Result::Ok(VarAccess::new(FilePosition::from_pair(&child), var));
                }
                Rule::func_expr => {
                    return convert_func_expr(
                        program,
                        scope,
                        preferred_output,
                        force_func_output,
                        child,
                    )
                }
                Rule::identifier => {
                    return Result::Ok(VarAccess::new(
                        FilePosition::from_pair(&child),
                        lookup_symbol_with_error(&program, scope, &child)?,
                    ))
                }
                Rule::expr => {
                    let output =
                        convert_expression(program, scope, preferred_output.clone(), true, child)?;
                    return Result::Ok(output);
                }
                Rule::array_literal => {
                    return convert_array_literal(program, scope, preferred_output, child);
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
    ) -> Result<VarAccess, CompileProblem> {
        unimplemented!();
    }

    fn convert_index_expr(
        program: &mut Program,
        scope: ScopeId,
        preferred_output: &Option<VarAccess>,
        force_func_output: bool,
        input: Pair<Rule>,
    ) -> Result<VarAccess, CompileProblem> {
        let mut iter = input.into_inner();
        let mut access = convert_expr_part_1(
            program,
            scope,
            &None,
            true,
            iter.next().expect("Required by grammer."),
        )?;
        for child in iter {
            match child.as_rule() {
                Rule::expr_part_1 => unreachable!("Already dealt with above."),
                Rule::expr => {
                    access.add_index(convert_expression(program, scope, None, true, child)?)
                }
                _ => unreachable!("Grammar specifies no other children."),
            }
        }
        Result::Ok(access)
    }

    fn convert_expr_part(
        program: &mut Program,
        scope: ScopeId,
        preferred_output: &Option<VarAccess>,
        force_func_output: bool,
        input: Pair<Rule>,
    ) -> Result<VarAccess, CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr_part_1 => {
                    return convert_expr_part_1(
                        program,
                        scope,
                        preferred_output,
                        force_func_output,
                        child,
                    );
                }
                Rule::index_expr => {
                    return convert_index_expr(
                        program,
                        scope,
                        preferred_output,
                        force_func_output,
                        child,
                    );
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

    fn operator_to_op_fn(operator: &Operator, blt: &Builtins) -> VariableId {
        if operator.id == ADD.id {
            blt.add_func
        } else if operator.id == SUBTRACT.id {
            blt.sub_func
        } else if operator.id == MULTIPLY.id {
            blt.mul_func
        } else if operator.id == DIVIDE.id {
            blt.div_func
        } else if operator.id == INT_DIV.id {
            blt.int_div_func
        } else if operator.id == MODULO.id {
            blt.mod_func
        } else if operator.id == POWER.id {
            blt.pow_func
        } else if operator.id == LTE.id {
            blt.lte_func
        } else if operator.id == LT.id {
            blt.lt_func
        } else if operator.id == GTE.id {
            blt.gte_func
        } else if operator.id == GT.id {
            blt.gt_func
        } else if operator.id == EQ.id {
            blt.eq_func
        } else if operator.id == NEQ.id {
            blt.neq_func
        } else if operator.id == BAND.id {
            blt.band_func
        } else if operator.id == BXOR.id {
            blt.bxor_func
        } else if operator.id == BOR.id {
            blt.bor_func
        } else if operator.id == AND.id {
            blt.and_func
        } else if operator.id == XOR.id {
            blt.xor_func
        } else if operator.id == OR.id {
            blt.or_func
        } else {
            unreachable!();
        }
    }

    fn convert_expression(
        program: &mut Program,
        scope: ScopeId,
        // If Some is provided, the return value will be identical to the var access provided.
        preferred_output: Option<VarAccess>,
        force_func_output: bool,
        input: Pair<Rule>,
    ) -> Result<VarAccess, CompileProblem> {
        let mut operand_stack = Vec::with_capacity(64);
        let mut operator_stack = Vec::with_capacity(64);
        operator_stack.push(SENTINEL);

        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr_part => {
                    let result = convert_expr_part(
                        program,
                        scope,
                        &preferred_output,
                        force_func_output,
                        child,
                    )?;
                    operand_stack.push(result);
                }
                Rule::operator => {
                    let op_str = child.as_str();
                    let operator = op_str_to_operator(op_str);
                    // TODO: Implement right-associative operators.
                    loop {
                        let top_op_prec = operator_stack.last().cloned().unwrap().precedence;
                        if operator.precedence >= top_op_prec {
                            operator_stack.push(operator);
                            break;
                        } else {
                            let top_op = operator_stack.pop().unwrap();
                            let func = operator_to_op_fn(&top_op, program.get_builtins());
                            // TODO: Real position.
                            let var = program
                                .make_intermediate_auto_var(scope, FilePosition::placeholder());
                            // TODO: Real position
                            let output = VarAccess::new(FilePosition::placeholder(), var);
                            // TODO: Real position.
                            let mut call = FuncCall::new(func, FilePosition::placeholder());
                            // Popping reverses the order, hence this is necessary.
                            let other = operand_stack.pop();
                            call.add_input(operand_stack.pop().unwrap());
                            call.add_input(other.unwrap());
                            call.add_output(output);
                            program.add_func_call(scope, call)?;
                            // TODO: real position.
                            operand_stack.push(VarAccess::new(FilePosition::placeholder(), var));
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        // Whenever we have an expression that has at least one operator, it is
        // guaranteed that all the input will be consumed before all the
        // operators have been popped. The loop below this statement handles
        // that case. This if statement is here to ensure that expressions with
        // no operators still get incorporated into the program. Without it,
        // the result of whatever the single term is would not be copied to the
        // output variable.
        if operator_stack.len() == 1 {
            return match preferred_output {
                Option::Some(final_output) => {
                    let expression_output = operand_stack.pop().unwrap();
                    if expression_output != final_output {
                        // TODO: Real position.
                        let mut call = FuncCall::new(
                            program.get_builtins().copy_func,
                            FilePosition::placeholder(),
                        );
                        call.add_input(expression_output);
                        call.add_output(final_output.clone());
                        program.add_func_call(scope, call)?;
                    }
                    Result::Ok(final_output)
                }
                Option::None => Result::Ok(operand_stack.pop().unwrap()),
            };
        }

        // The last operator is the sentinel, we don't actually want to pop it.
        while operator_stack.len() > 1 {
            let top_op = operator_stack.pop().unwrap();
            let func = operator_to_op_fn(&top_op, program.get_builtins());
            // If the length is 1, then we popped the last operator, so we
            // should output the result to the output given to us. Otherwise,
            // make a new intermediate variable.
            let output = if operator_stack.len() == 1 {
                match &preferred_output {
                    Option::Some(final_output) => final_output.clone(),
                    Option::None => {
                        // TODO: Real position
                        let var =
                            program.make_intermediate_auto_var(scope, FilePosition::placeholder());
                        // TODO: Real position
                        VarAccess::new(FilePosition::placeholder(), var)
                    }
                }
            } else {
                // TODO: Real position
                let var = program.make_intermediate_auto_var(scope, FilePosition::placeholder());
                // TODO: Real position
                VarAccess::new(FilePosition::placeholder(), var)
            };
            // TODO: Real position.
            let mut call = FuncCall::new(func, FilePosition::placeholder());
            // Popping reverses the order, hence this is necessary.
            let other = operand_stack.pop();
            call.add_input(operand_stack.pop().unwrap());
            call.add_input(other.unwrap());
            call.add_output(output.clone());
            program.add_func_call(scope, call)?;
            // The last operator is the sentinel.
            if operator_stack.len() == 1 {
                return Result::Ok(output);
            } else {
                operand_stack.push(output);
            }
        }
        unreachable!();
    }

    fn convert_assign_expr(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<VarAccess, CompileProblem> {
        let input_pos = FilePosition::from_pair(&input);
        let mut input_iter = input.into_inner();
        let child = input_iter.next().expect("Identifier required by grammar.");
        let mut result = VarAccess::new(
            input_pos,
            lookup_symbol_with_error(&program, scope, &child)?,
        );
        for child in input_iter {
            match child.as_rule() {
                Rule::identifier => unreachable!("Already handled above."),
                Rule::assign_array_access => result.add_index(convert_expression(
                    program,
                    scope,
                    None,
                    true,
                    child.into_inner().next().expect("Required by grammar."),
                )?),
                _ => unreachable!("Grammar specifies no other children."),
            }
        }
        Result::Ok(result)
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
        func: &mut FunctionData,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        for child in input.into_inner() {
            let new_input = parse_named_function_parameter(program, func_scope, child)?;
            program.borrow_scope_mut(func_scope).add_input(new_input);
        }
        Result::Ok(())
    }

    fn add_function_outputs(
        program: &mut Program,
        func: &mut FunctionData,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        for child in input.into_inner() {
            let new_output = parse_named_function_parameter(program, func_scope, child)?;
            program.borrow_scope_mut(func_scope).add_output(new_output);
        }
        Result::Ok(())
    }

    fn add_function_output(
        program: &mut Program,
        func: &mut FunctionData,
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
        program.borrow_scope_mut(func_scope).add_output(new_output);
        Result::Ok(())
    }

    fn convert_function_signature(
        program: &mut Program,
        func: &mut FunctionData,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::function_inputs => add_function_inputs(program, func, func_scope, child)?,
                Rule::function_outputs => add_function_outputs(program, func, func_scope, child)?,
                Rule::single_function_output => {
                    add_function_output(program, func, func_scope, child)?
                }
                _ => unreachable!(),
            }
        }
        Result::Ok(())
    }

    fn convert_returnable_code_block(
        program: &mut Program,
        scope: ScopeId,
        return_var: Option<VarAccess>,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::statement => convert_statement(program, scope, child)?,
                Rule::expr => {
                    convert_expression(program, scope, return_var.clone(), true, child)?;
                }
                _ => unreachable!(),
            }
        }
        Result::Ok(())
    }

    fn convert_function_definition(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let mut name = Option::None;
        let func_scope = program.create_child_scope(scope);
        let mut function = FunctionData::new(func_scope, FilePosition::placeholder());
        let mut real_header_position = FilePosition::start_at(&input);
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::identifier => {
                    real_header_position.include(&child);
                    name = Option::Some(child.as_str());
                }
                Rule::function_signature => {
                    real_header_position.include(&child);
                    convert_function_signature(program, &mut function, func_scope, child)?;
                }
                Rule::returnable_code_block => {
                    let output_value = program
                        .borrow_scope_mut(func_scope)
                        .get_single_output()
                        .and_then(|id: VariableId| -> Option<VarAccess> {
                            Option::Some(VarAccess::new(FilePosition::from_pair(&child), id))
                        });
                    function.set_header(real_header_position);
                    // So that code inside the body can refer to the function.
                    // If name is None, there is a bug in the parser.
                    program.adopt_and_define_symbol(
                        scope,
                        name.expect("Grammar requires a name."),
                        Variable::function_def(function),
                    );
                    convert_returnable_code_block(program, func_scope, output_value, child)?;
                    // This branch arm can only be called once but I don't know how to tell rustc that,
                    // so we use a break statement for that purpose. Since the code block is the last element
                    // parsed anyway, it doesn't change how the code works.
                    break;
                }
                _ => unreachable!(),
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
        let mut name = Option::None;
        let input_pos = FilePosition::from_pair(&input);
        let variable_id =
            program.adopt_variable(Variable::variable(input_pos.clone(), data_type, None));
        let mut variable_position = FilePosition::placeholder();
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::identifier => {
                    name = Option::Some(child.as_str());
                    variable_position = FilePosition::from_pair(&child);
                }
                Rule::expr => {
                    convert_expression(
                        program,
                        scope,
                        Option::Some(VarAccess::new(variable_position.clone(), variable_id)),
                        true,
                        child,
                    )
                    .map_err(|mut err| {
                        problem::hint_encountered_while_parsing(
                            "initial value for a variable",
                            input_pos.clone(),
                            &mut err,
                        );
                        err
                    })?;
                }
                _ => unreachable!(),
            }
        }
        program
            .borrow_scope_mut(scope)
            .define_symbol(name.unwrap(), variable_id);
        Result::Ok(())
    }

    fn convert_empty_variable(
        program: &mut Program,
        scope: ScopeId,
        data_type: DataType,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let mut name = Option::None;
        let input_pos = FilePosition::from_pair(&input);
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::identifier => name = Option::Some(child.as_str()),
                _ => unreachable!(),
            }
        }
        program.adopt_and_define_symbol(
            scope,
            name.unwrap(),
            Variable::variable(input_pos, data_type, None),
        );
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
        let type_variable = program.borrow_variable(type_variable_id);
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
        // We have to store the sizes and process them later because the base type comes after all
        // the sizes.
        let mut sizes = Vec::new();
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr => sizes.push(convert_expression(program, scope, None, true, child)?),
                Rule::basic_data_type => {
                    // Array data type stores sizes in the same order as the grammar, biggest to
                    // smallest.
                    let data_type = convert_basic_data_type(program, scope, child)?;
                    data_type.wrap_with_sizes(sizes);
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
        let mut data_type = Option::None;
        if program.borrow_scope(scope).get_parent().is_some() {
            return Result::Err(problem::io_inside_function(FilePosition::from_pair(&input)));
        }
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::data_type => {
                    data_type = Option::Some(convert_data_type(program, scope, child)?)
                }
                Rule::identifier => {
                    let new_input = program.adopt_and_define_symbol(
                        scope,
                        child.as_str(),
                        Variable::variable(
                            FilePosition::from_pair(&child),
                            data_type
                                .as_ref()
                                .expect("Grammar requires data type before identifier.")
                                .clone(),
                            None,
                        ),
                    );
                    program.borrow_scope_mut(scope).add_input(new_input);
                }
                _ => unreachable!(),
            }
        }
        Result::Ok(())
    }

    fn convert_output_variable_statement(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let mut data_type = Option::None;
        if program.borrow_scope(scope).get_parent().is_some() {
            return Result::Err(problem::io_inside_function(FilePosition::from_pair(&input)));
        }
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::data_type => {
                    data_type = Option::Some(convert_data_type(program, scope, child)?)
                }
                Rule::identifier => {
                    let new_output = program.adopt_and_define_symbol(
                        scope,
                        child.as_str(),
                        Variable::variable(
                            FilePosition::from_pair(&child),
                            data_type
                                .as_ref()
                                .expect("Grammar requires data type before identifier.")
                                .clone(),
                            None,
                        ),
                    );
                    program.borrow_scope_mut(scope).add_output(new_output);
                }
                _ => unreachable!(),
            }
        }
        Result::Ok(())
    }

    fn convert_create_variable_statement(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> Result<(), CompileProblem> {
        let mut data_type = Option::None;
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::data_type => {
                    data_type = Option::Some(convert_data_type(program, scope, child)?)
                }
                Rule::assigned_variable => convert_assigned_variable(
                    program,
                    scope,
                    data_type
                        .clone()
                        .expect("Grammar requires data type before variable."),
                    child,
                )?,
                Rule::empty_variable => convert_empty_variable(
                    program,
                    scope,
                    data_type
                        .clone()
                        .expect("Grammar requires data type before variable."),
                    child,
                )?,
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
        let mut index = 0;
        let func = program
            .lookup_and_clone_parent_function(scope)
            .ok_or_else(|| problem::return_from_root(FilePosition::from_pair(&input)))?;
        let inputs = program
            .borrow_scope(func.get_body())
            .borrow_inputs()
            .clone();
        let outputs = program
            .borrow_scope(func.get_body())
            .borrow_outputs()
            .clone();
        // In case we need to make an error, we can't borrow input once we enter the loop because
        // the loop consumes it.
        let statement_position = FilePosition::from_pair(&input);
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr => {
                    if index >= outputs.len() {
                        return Result::Err(problem::extra_return_value(
                            statement_position,
                            FilePosition::from_pair(&child),
                            func.get_header().clone(),
                            outputs.len(),
                        ));
                    }
                    convert_expression(
                        program,
                        scope,
                        Option::Some(VarAccess::new(
                            FilePosition::from_pair(&child),
                            outputs[index],
                        )),
                        true,
                        child,
                    )
                    .map_err(|mut err| {
                        problem::hint_encountered_while_parsing(
                            "a return statement",
                            statement_position.clone(),
                            &mut err,
                        );
                        err
                    })?;
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
        program.add_func_call(
            scope,
            FuncCall::new(program.get_builtins().return_func, statement_position),
        )?;
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
                    let mut iter = child.into_inner();
                    let assign_expr = iter.next().unwrap();
                    debug_assert!(match assign_expr.as_rule() {
                        Rule::assign_expr => true,
                        _ => false,
                    });
                    let output = convert_assign_expr(program, scope, assign_expr)?;
                    let expr = iter.next().unwrap();
                    debug_assert!(match expr.as_rule() {
                        Rule::expr => true,
                        _ => false,
                    });
                    convert_expression(program, scope, Option::Some(output), true, expr)?;
                }
                Rule::expr => {
                    convert_expression(program, scope, None, false, child)?;
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
