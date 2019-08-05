extern crate pest;

use pest::error::Error;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct WaveguideParser;

pub type ParseResult<'a> = Pairs<'a, Rule>;
pub type ParseError = Error<Rule>;

pub fn parse(text: &str) -> Result<ParseResult, ParseError> {
    WaveguideParser::parse(Rule::root, text)
}

// We have to put this here because pest does not allow us to export the auto
// generated Rule enum.
pub mod convert {
    use super::*;
    use crate::vague::*;

    fn parse_dec_int(input: &str) -> i64 {
        input.replace("_", "").parse().unwrap()
    }

    fn convert_func_expr_input_list(
        program: &mut Program,
        scope: ScopeId,
        func_call: &mut FuncCall,
        input: Pair<Rule>,
    ) {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr => {
                    let arg_var = VarAccess::new(program.make_intermediate_auto_var(scope));
                    convert_expression(program, scope, arg_var.clone(), child);
                    func_call.add_input(arg_var);
                }
                _ => unreachable!(),
            }
        }
    }

    fn convert_func_expr_output_list(
        program: &mut Program,
        scope: ScopeId,
        func_call: &mut FuncCall,
        input: Pair<Rule>,
    ) {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::func_output_return_inline => unimplemented!(),
                Rule::func_output_var_dec => unimplemented!(),
                Rule::assign_expr => {
                    let into = convert_assign_expr(program, scope, child);
                    func_call.add_output(into);
                }
                _ => unreachable!(),
            }
        }
    }

    fn convert_func_expr(program: &mut Program, scope: ScopeId, input: Pair<Rule>) -> VarAccess {
        let mut output_var = Option::None;
        let mut func_call = Option::None;
        for child in input.into_inner() {
            match child.as_rule() {
                // TODO: Real error message.
                Rule::identifier => {
                    func_call = Option::Some(FuncCall::new(
                        program.lookup_symbol(scope, child.as_str()).unwrap(),
                    ))
                }
                Rule::func_input_list => {
                    convert_func_expr_input_list(program, scope, func_call.as_mut().unwrap(), child)
                }
                Rule::func_output_list => convert_func_expr_output_list(
                    program,
                    scope,
                    func_call.as_mut().unwrap(),
                    child,
                ),
                Rule::func_lambda => unimplemented!(),
                Rule::lambda_adjective => unimplemented!(),
                _ => unreachable!(),
            }
        }
        program.add_func_call(scope, func_call.unwrap());
        match output_var {
            Option::Some(value) => VarAccess::new(value),
            Option::None => {
                VarAccess::new(program.adopt_and_define_intermediate(
                    scope,
                    make_var(program.get_builtins().void_type),
                ))
            }
        }
    }

    fn convert_expr_part_1(program: &mut Program, scope: ScopeId, input: Pair<Rule>) -> VarAccess {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::bin_int => unimplemented!(),
                Rule::hex_int => unimplemented!(),
                Rule::oct_int => unimplemented!(),
                Rule::legacy_oct_int => unimplemented!(),
                Rule::dec_int => {
                    let val = Entity::IntLiteral(parse_dec_int(child.as_str()));
                    return VarAccess::new(program.adopt_entity(val));
                }
                Rule::float => unimplemented!(),
                Rule::func_expr => return convert_func_expr(program, scope, child),
                // TODO: Real error message.
                Rule::identifier => {
                    return VarAccess::new(
                        program
                            .lookup_symbol(scope, child.as_str())
                            .expect("Symbol not defined!"),
                    )
                }
                Rule::expr => {
                    let output = VarAccess::new(program.make_intermediate_auto_var(scope));
                    convert_expression(program, scope, output.clone(), child);
                    return output;
                }
                Rule::array_literal => unimplemented!(),
                _ => unreachable!(),
            }
        }
        unreachable!();
    }

    fn convert_negate(program: &mut Program, scope: ScopeId, input: Pair<Rule>) -> VarAccess {
        unimplemented!();
    }

    fn convert_expr_part(program: &mut Program, scope: ScopeId, input: Pair<Rule>) -> VarAccess {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::negate => {
                    return convert_negate(program, scope, child);
                }
                Rule::expr_part_1 => {
                    return convert_expr_part_1(program, scope, child);
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

    fn operator_to_op_fn(operator: &Operator, blt: &Builtins) -> EntityId {
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
        final_output: VarAccess,
        input: Pair<Rule>,
    ) {
        let mut operand_stack = Vec::with_capacity(64);
        let mut operator_stack = Vec::with_capacity(64);
        operator_stack.push(SENTINEL);

        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr_part => {
                    let result = convert_expr_part(program, scope, child);
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
                            let var = program.make_intermediate_auto_var(scope);
                            let output = VarAccess::new(var);
                            let mut call = FuncCall::new(func);
                            // Popping reverses the order, hence this is necessary.
                            let other = operand_stack.pop();
                            call.add_input(operand_stack.pop().unwrap());
                            call.add_input(other.unwrap());
                            call.add_output(output);
                            program.add_func_call(scope, call);
                            operand_stack.push(VarAccess::new(var));
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
            let mut call = FuncCall::new(program.get_builtins().copy_func);
            call.add_input(operand_stack.pop().unwrap());
            call.add_output(final_output);
            program.add_func_call(scope, call);
            return;
        }

        // The last operator is the sentinel, we don't actually want to pop it.
        while operator_stack.len() > 1 {
            let top_op = operator_stack.pop().unwrap();
            let func = operator_to_op_fn(&top_op, program.get_builtins());
            // If the length is 1, then we popped the last operator, so we
            // should output the result to the output given to us. Otherwise,
            // make a new intermediate variable.
            let output = if operator_stack.len() == 1 {
                final_output.clone()
            } else {
                let var = program.make_intermediate_auto_var(scope);
                VarAccess::new(var)
            };
            let mut call = FuncCall::new(func);
            // Popping reverses the order, hence this is necessary.
            let other = operand_stack.pop();
            call.add_input(operand_stack.pop().unwrap());
            call.add_input(other.unwrap());
            call.add_output(output.clone());
            program.add_func_call(scope, call);
            operand_stack.push(output);
        }
    }

    fn convert_assign_expr(program: &mut Program, scope: ScopeId, input: Pair<Rule>) -> VarAccess {
        let mut result = Option::None;
        for child in input.into_inner() {
            match child.as_rule() {
                // TODO: Real error message.
                Rule::identifier => {
                    result = Option::Some(VarAccess::new(
                        program.lookup_symbol(scope, child.as_str()).unwrap(),
                    ))
                }
                Rule::assign_array_access => unimplemented!(),
                _ => unreachable!(),
            }
        }
        result.unwrap()
    }

    // Creates a variable, returns its id.
    fn parse_named_function_parameter(
        program: &mut Program,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) -> EntityId {
        let mut name = Option::None;
        let mut data_type = Option::None;
        for part in input.into_inner() {
            match part.as_rule() {
                Rule::data_type => {
                    data_type = Option::Some(convert_data_type(program, func_scope, part))
                }
                Rule::identifier => name = Option::Some(part.as_str()),
                _ => unreachable!(),
            }
        }
        let variable = VariableEntity::new(data_type.unwrap());
        program.adopt_and_define_symbol(func_scope, name.unwrap(), Entity::Variable(variable))
    }

    fn add_function_inputs(
        program: &mut Program,
        func: &mut FunctionEntity,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) {
        for child in input.into_inner() {
            func.add_input(parse_named_function_parameter(program, func_scope, child));
        }
    }

    fn add_function_outputs(
        program: &mut Program,
        func: &mut FunctionEntity,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) {
        for child in input.into_inner() {
            func.add_output(parse_named_function_parameter(program, func_scope, child));
        }
    }

    fn add_function_output(
        program: &mut Program,
        func: &mut FunctionEntity,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) {
        let mut data_type = Option::None;
        for part in input.into_inner() {
            match part.as_rule() {
                Rule::data_type => {
                    data_type = Option::Some(convert_data_type(program, func_scope, part))
                }
                _ => unreachable!(),
            }
        }
        let variable = VariableEntity::new(data_type.unwrap());
        func.add_output(program.adopt_and_define_symbol(
            func_scope,
            "!return_value",
            Entity::Variable(variable),
        ));
    }

    fn convert_function_signature(
        program: &mut Program,
        func: &mut FunctionEntity,
        func_scope: ScopeId,
        input: Pair<Rule>,
    ) {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::function_inputs => add_function_inputs(program, func, func_scope, child),
                Rule::function_outputs => add_function_outputs(program, func, func_scope, child),
                Rule::single_function_output => {
                    add_function_output(program, func, func_scope, child)
                }
                _ => unreachable!(),
            }
        }
    }

    fn convert_returnable_code_block(
        program: &mut Program,
        scope: ScopeId,
        return_var: Option<VarAccess>,
        input: Pair<Rule>,
    ) {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::statement => convert_statement(program, scope, child),
                Rule::expr => {
                    let result_var = match return_var.as_ref() {
                        Option::Some(access) => access.clone(),
                        Option::None => VarAccess::new(program.make_intermediate_auto_var(scope)),
                    };
                    convert_expression(program, scope, result_var, child);
                }
                _ => unreachable!(),
            }
        }
    }

    fn convert_function_definition(program: &mut Program, scope: ScopeId, input: Pair<Rule>) {
        let mut name = Option::None;
        let func_scope = program.create_child_scope(scope);
        let mut function = FunctionEntity::new(func_scope);
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::identifier => name = Option::Some(child.as_str()),
                Rule::function_signature => {
                    convert_function_signature(program, &mut function, func_scope, child);
                }
                Rule::returnable_code_block => {
                    let output_value = function.get_single_output().and_then(
                        |id: EntityId| -> Option<VarAccess> { Option::Some(VarAccess::new(id)) },
                    );
                    // So that code inside the body can refer to the function.
                    // If name is None, there is a bug in the parser.
                    program.adopt_and_define_symbol(
                        scope,
                        name.unwrap(),
                        Entity::Function(function),
                    );
                    convert_returnable_code_block(program, func_scope, output_value, child);
                    // This branch arm can only be called once but I don't know how to tell rustc that,
                    // so we use a break statement for that purpose. Since the code block is the last element
                    // parsed anyway, it doesn't change how the code works.
                    break;
                }
                _ => unreachable!(),
            }
        }
    }

    // TODO: Take in data type.
    fn convert_assigned_variable(
        program: &mut Program,
        scope: ScopeId,
        data_type: EntityId,
        input: Pair<Rule>,
    ) {
        let mut name = Option::None;
        let variable = VariableEntity::new(data_type);
        let id = program.adopt_entity(Entity::Variable(variable));
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::identifier => name = Option::Some(child.as_str()),
                Rule::expr => convert_expression(program, scope, VarAccess::new(id), child),
                _ => unreachable!(),
            }
        }
        program.define_symbol(scope, name.unwrap(), id);
    }

    // TODO: Take in data type.
    fn convert_empty_variable(
        program: &mut Program,
        scope: ScopeId,
        data_type: EntityId,
        input: Pair<Rule>,
    ) {
        let mut name = Option::None;
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::identifier => name = Option::Some(child.as_str()),
                _ => unreachable!(),
            }
        }
        let variable = VariableEntity::new(data_type);
        let id = program.adopt_and_define_symbol(scope, name.unwrap(), Entity::Variable(variable));
    }

    fn convert_basic_data_type(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> EntityId {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::named_data_type => {
                    for sub_child in child.into_inner() {
                        match sub_child.as_rule() {
                            // TODO: Real error.
                            Rule::identifier => {
                                return program
                                    .lookup_symbol(scope, sub_child.as_str())
                                    .expect("Datatype not defined!")
                            }
                            _ => unreachable!(),
                        }
                    }
                    unreachable!();
                }
                Rule::dynamic_data_type => unimplemented!(),
                _ => unreachable!(),
            }
        }
        unreachable!();
    }

    fn convert_array_data_type(
        program: &mut Program,
        scope: ScopeId,
        input: Pair<Rule>,
    ) -> EntityId {
        unimplemented!();
    }

    fn convert_data_type(program: &mut Program, scope: ScopeId, input: Pair<Rule>) -> EntityId {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::array_data_type => return convert_array_data_type(program, scope, child),
                Rule::basic_data_type => return convert_basic_data_type(program, scope, child),
                _ => unreachable!(),
            }
        }
        unreachable!();
    }

    fn convert_create_variable_statement(program: &mut Program, scope: ScopeId, input: Pair<Rule>) {
        let mut data_type = Option::None;
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::data_type => {
                    data_type = Option::Some(convert_data_type(program, scope, child))
                } // TODO: Include data type.
                Rule::assigned_variable => {
                    convert_assigned_variable(program, scope, data_type.unwrap(), child)
                }
                Rule::empty_variable => {
                    convert_empty_variable(program, scope, data_type.unwrap(), child)
                }
                _ => unreachable!(),
            }
        }
    }

    fn convert_return_statement(program: &mut Program, scope: ScopeId, input: Pair<Rule>) {
        let mut index = 0;
        // TODO: Real error if we aren't inside a function.
        let func = program.lookup_and_clone_parent_function(scope).unwrap();
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::expr => {
                    // TODO: Real error if the programmer specified the wrong number of return values.
                    convert_expression(
                        program,
                        scope,
                        VarAccess::new(func.get_output(index)),
                        child,
                    );
                    index += 1;
                }
                _ => unreachable!(),
            }
        }
        program.add_func_call(scope, FuncCall::new(program.get_builtins().return_func));
    }

    fn convert_statement(program: &mut Program, scope: ScopeId, input: Pair<Rule>) {
        for child in input.into_inner() {
            match child.as_rule() {
                Rule::function_definition => convert_function_definition(program, scope, child),
                Rule::code_block => unimplemented!(),
                Rule::return_statement => convert_return_statement(program, scope, child),
                Rule::create_variable_statement => {
                    convert_create_variable_statement(program, scope, child)
                }
                Rule::assign_statement => {
                    let mut iter = child.into_inner();
                    let assign_expr = iter.next().unwrap();
                    debug_assert!(match assign_expr.as_rule() {
                        Rule::assign_expr => true,
                        _ => false,
                    });
                    let output = convert_assign_expr(program, scope, assign_expr);
                    let expr = iter.next().unwrap();
                    debug_assert!(match expr.as_rule() {
                        Rule::expr => true,
                        _ => false,
                    });
                    convert_expression(program, scope, output, expr);
                }
                Rule::expr => {
                    let result = VarAccess::new(program.make_intermediate_auto_var(scope));
                    convert_expression(program, scope, result, child);
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn convert_ast_to_vague(input: &mut ParseResult) -> Program {
        let root = input.next().unwrap();
        let mut program = Program::new();
        let scope = program.get_root_scope();

        for statement in root.into_inner() {
            match statement.as_rule() {
                Rule::EOI => continue,
                Rule::statement => convert_statement(&mut program, scope, statement),
                _ => unreachable!(),
            }
        }

        program
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid(text: &str) -> bool {
        return match parse(text) {
            Ok(pairs) => {
                println!("{:#?}", pairs);
                true
            }
            Err(error) => {
                println!("{:#?}", error);
                false
            }
        };
    }

    #[test]
    fn basic_function_call() {
        assert!(is_valid("func();"));
        assert!(is_valid("test_function_12938 (  )   ;"));

        assert!(!is_valid("func(;"));
        assert!(!is_valid("func);"));
        assert!(!is_valid("12039821();"));
    }

    #[test]
    fn input_function_call() {
        assert!(is_valid("func(12);"));
        assert!(is_valid("func(12, 34  , 120);"));
    }

    #[test]
    fn output_function_call() {
        assert!(is_valid("func:();"));
        assert!(is_valid("func:(asdf);"));
        assert!(is_valid("func:(out1, out2  , out3);"));

        assert!(!is_valid("func:(123);"));
    }

    #[test]
    fn input_output_function_call() {
        assert!(is_valid("func():();"));
        assert!(is_valid("func(in1):(out1);"));
        assert!(is_valid("func(in1, in2):(out1, out2);"));

        assert!(!is_valid("func(in1, in2):(out1, 12);"));
    }

    #[test]
    fn lambda_function_call() {
        assert!(is_valid("func { };"));
        assert!(is_valid("func(in1):(out1) { };"));
        assert!(is_valid("func(in1):(out1) { func(in1):(out1); };"));
        assert!(is_valid("func(in1):(out1) { func(in1) };"));
        assert!(is_valid("func(in1):(out1) { } { } { };"));

        assert!(!is_valid("{ func(); };"));
    }

    #[test]
    fn adjective_function_call() {
        // According to grammar specification, all function calls must specify
        // at least one of: input list, output list, or code block with no
        // preceding adjectives. This makes the grammar unambiguous
        assert!(is_valid("func {} adj1;"));
        assert!(is_valid("func() adj1;"));
        assert!(is_valid("func:() adj1;"));

        // This is, so far, the only syntactically invalid type of function call
        // which does not have any alternate meaning. (E.G. func adj1; resolves
        // to a variable declaration, so it should be positively tested for in
        // another test.)
        assert!(!is_valid("func adj1 { };"));
    }

    #[test]
    fn variable_declaration() {
        assert!(is_valid("Int a;"));
        assert!(is_valid("Int a = 12;"));
        assert!(is_valid("Int a, b;"));
        assert!(is_valid("Int a = 12, b = 13;"));
    }

    #[test]
    fn variable_assignment() {
        assert!(is_valid("a;"));
        assert!(is_valid("a = 12;"));
    }

    #[test]
    fn array_declaration() {
        assert!(is_valid("[4]Int a;"));
        assert!(is_valid("[4][3]Int a;"));
        assert!(is_valid("[4]Int a = [1, 2, 3, 4];"));
    }

    #[test]
    fn arithmetic() {
        assert!(is_valid("a = 12 + 34;"));
        assert!(is_valid("a = 12 - 34;"));
        assert!(is_valid("a = 12 * 34;"));
        assert!(is_valid("a = 12 ** 34;"));
        assert!(is_valid("a = 12 / 34;"));
        assert!(is_valid("a = 12 // 34;"));
        assert!(is_valid("a = 12 % 34;"));
    }

    #[test]
    fn logic() {
        assert!(is_valid("a = 12 and 34;"));
        assert!(is_valid("a = 12 or 34;"));
        assert!(is_valid("a = 12 xor 34;"));
        assert!(is_valid("a = 12 nand 34;"));
        assert!(is_valid("a = 12 nor 34;"));
        assert!(is_valid("a = 12 xnor 34;"));
    }

    #[test]
    fn bitwise_logic() {
        assert!(is_valid("a = 12 band 34;"));
        assert!(is_valid("a = 12 bor 34;"));
        assert!(is_valid("a = 12 bxor 34;"));
        assert!(is_valid("a = 12 bnand 34;"));
        assert!(is_valid("a = 12 bnor 34;"));
        assert!(is_valid("a = 12 bxnor 34;"));
    }

    #[test]
    fn comparison() {
        assert!(is_valid("a = 12 == 34;"));
        assert!(is_valid("a = 12 != 34;"));
        assert!(is_valid("a = 12 >= 34;"));
        assert!(is_valid("a = 12 <= 34;"));
        assert!(is_valid("a = 12 > 34;"));
        assert!(is_valid("a = 12 < 34;"));
    }

    #[test]
    fn literals() {
        assert!(is_valid("a = 12;"));
        assert!(is_valid("a = 12.0;"));
        assert!(is_valid("a = 0.01;"));
        assert!(is_valid("a = .01;"));
        assert!(is_valid("a = -4;"));
        assert!(is_valid("a = -4.3e1;"));
        assert!(is_valid("a = -4.3e+1;"));
        assert!(is_valid("a = -4.3e-1;"));
        assert!(is_valid("a = -3e-1;"));
        assert!(is_valid("a = .1e-1;"));

        assert!(is_valid("a = -01_234567;"));
        assert!(is_valid("a = -0o1_234567;"));
        assert!(is_valid("a = -0x9_ABCDEFabcdef;"));
        assert!(is_valid("a = -0b0_1;"));
        assert!(is_valid("a = -0b0_1;"));

        assert!(!is_valid("a = 0b2"));
        assert!(!is_valid("a = 0o8"));
        assert!(!is_valid("a = 08"));
        assert!(!is_valid("a = 0xG"));
    }

    #[test]
    fn function_definition() {
        assert!(is_valid("fn main { }"));
        assert!(is_valid("fn main() { }"));
        assert!(is_valid("fn main:() { }"));
        assert!(is_valid("fn main:(Int a) { }"));
        assert!(is_valid("fn main:Int { }"));
    }
}
