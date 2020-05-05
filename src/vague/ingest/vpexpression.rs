use super::problems;
use super::VagueIngester;
use crate::ast::structure as i;
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::vague::structure as o;

#[derive(Clone)]
enum Operator {
    Sentinel,
    Power,
    Multiply,
    Divide,
    Modulo,
    Add,
    Subtract,
    Lte,
    Lt,
    Gte,
    Gt,
    Eq,
    Neq,
    Band,
    Bor,
    Bxor,
    And,
    Or,
    Xor,
}

impl Operator {
    fn precedence(&self) -> u32 {
        match self {
            Self::Power => 19,
            Self::Multiply => 18,
            Self::Divide => 18,
            Self::Modulo => 18,
            Self::Add => 17,
            Self::Subtract => 17,
            Self::Band => 16,
            Self::Bxor => 15,
            Self::Bor => 14,
            Self::Lte => 13,
            Self::Lt => 13,
            Self::Gte => 13,
            Self::Gt => 13,
            Self::Eq => 13,
            Self::Neq => 13,
            Self::And => 12,
            Self::Xor => 11,
            Self::Or => 10,
            Self::Sentinel => 0,
        }
    }

    fn right_associative(&self) -> bool {
        match self {
            Self::Power => true,
            _ => false,
        }
    }

    fn bin_op(&self) -> o::BinaryOperator {
        match self {
            Self::Power => o::BinaryOperator::Power,
            Self::Multiply => o::BinaryOperator::Multiply,
            Self::Divide => o::BinaryOperator::Divide,
            Self::Modulo => o::BinaryOperator::Modulo,
            Self::Add => o::BinaryOperator::Add,
            Self::Subtract => o::BinaryOperator::Subtract,
            Self::Lte => o::BinaryOperator::LessThanOrEqual,
            Self::Lt => o::BinaryOperator::LessThan,
            Self::Gte => o::BinaryOperator::GreaterThanOrEqual,
            Self::Gt => o::BinaryOperator::GreaterThan,
            Self::Eq => o::BinaryOperator::Equal,
            Self::Neq => o::BinaryOperator::NotEqual,
            Self::Band => o::BinaryOperator::BAnd,
            Self::Bor => o::BinaryOperator::BOr,
            Self::Bxor => o::BinaryOperator::BXor,
            Self::And => o::BinaryOperator::And,
            Self::Or => o::BinaryOperator::Or,
            Self::Xor => o::BinaryOperator::Xor,
            Self::Sentinel => panic!(
                "Despite what Big Sentinel may have you think, Sentinel is not a real operator."
            ),
        }
    }
}

fn op_str_to_operator(op_str: &str) -> Operator {
    match op_str {
        "**" => Operator::Power,
        "+" => Operator::Add,
        "-" => Operator::Subtract,
        "*" => Operator::Multiply,
        "/" => Operator::Divide,
        "%" => Operator::Modulo,
        "<=" => Operator::Lte,
        "<" => Operator::Lt,
        ">=" => Operator::Gte,
        ">" => Operator::Gt,
        "==" => Operator::Eq,
        "!=" => Operator::Neq,
        "band" => Operator::Band,
        "bxor" => Operator::Bxor,
        "bor" => Operator::Bor,
        "and" => Operator::And,
        "xor" => Operator::Xor,
        "or" => Operator::Or,
        "bnand" => unimplemented!(),
        "bxnor" => unimplemented!(),
        "bnor" => unimplemented!(),
        "nand" => unimplemented!(),
        "xnor" => unimplemented!(),
        "nor" => unimplemented!(),
        _ => unreachable!(),
    }
}

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
    i64::from_str_radix(&input.replace("_", "")[2..], 8).expect("Grammar requires valid octal int.")
}

fn parse_legacy_oct_int(input: &str) -> i64 {
    // Slice trims off 0 at beginning.
    i64::from_str_radix(&input.replace("_", "")[1..], 8).expect("Grammar requires valid octal int.")
}

fn parse_bin_int(input: &str) -> i64 {
    // Slice trims off 0b at beginning.
    i64::from_str_radix(&input.replace("_", "")[2..], 2)
        .expect("Grammar requires valid binary int.")
}

impl VagueIngester {
    pub(super) fn convert_literal(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::literal);
        let position = FilePosition::from_pair(&node);
        let child = node.into_inner().next().expect("illegal grammar");

        let value = match child.as_rule() {
            i::Rule::bin_int => o::KnownData::Int(parse_bin_int(child.as_str())),
            i::Rule::oct_int => o::KnownData::Int(parse_oct_int(child.as_str())),
            i::Rule::dec_int => o::KnownData::Int(parse_dec_int(child.as_str())),
            i::Rule::hex_int => o::KnownData::Int(parse_hex_int(child.as_str())),
            i::Rule::legacy_oct_int => o::KnownData::Int(parse_legacy_oct_int(child.as_str())),
            i::Rule::float => o::KnownData::Float(parse_float(child.as_str())),
            _ => unreachable!("illegal grammar"),
        };
        Ok(o::VPExpression::Literal(value, position))
    }

    pub(super) fn convert_macro_call_input_list(
        &mut self,
        node: i::Node,
    ) -> Result<Vec<o::VPExpression>, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::macro_call_input_list);
        let mut inputs = Vec::new();
        for child in node.into_inner() {
            inputs.push(self.convert_vpe(child)?);
        }
        Ok(inputs)
    }

    pub(super) fn convert_macro_call_output_list(
        &mut self,
        node: i::Node,
    ) -> Result<Vec<o::FuncCallOutput>, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::macro_call_output_list);
        let mut outputs = Vec::new();
        for child in node.into_inner() {
            let output = match child.as_rule() {
                i::Rule::vce => o::FuncCallOutput::VCExpression(Box::new(self.convert_vce(child)?)),
                i::Rule::inline_output => {
                    o::FuncCallOutput::InlineReturn(FilePosition::from_pair(&child))
                }
                _ => unreachable!("illegal grammar"),
            };
            outputs.push(output);
        }
        Ok(outputs)
    }

    pub(super) fn convert_macro_call(
        &mut self,
        node: i::Node,
        require_inline_output: bool,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::macro_call);
        let position = FilePosition::from_pair(&node);
        let mut children = node.into_inner();
        let name_node = children.next().expect("illegal grammar");
        let input_list_node = children.next().expect("illegal grammar");
        let maybe_output_list_node = children.next();

        let input_list = self.convert_macro_call_input_list(input_list_node)?;
        let output_list = if let Some(output_list_node) = maybe_output_list_node {
            let output_list_position = FilePosition::from_pair(&output_list_node);
            let output_list = self.convert_macro_call_output_list(output_list_node)?;
            let num_inline_outputs = output_list.iter().fold(0, |counter, item| {
                if let o::FuncCallOutput::InlineReturn(..) = item {
                    counter + 1
                } else {
                    counter
                }
            });
            if num_inline_outputs > 1 {
                return Err(problems::too_many_inline_returns(
                    position,
                    output_list_position,
                    num_inline_outputs,
                ));
            }
            if require_inline_output && num_inline_outputs != 1 {
                return Err(problems::missing_inline_return(
                    position,
                    output_list_position,
                ));
            }
            output_list
        } else if require_inline_output {
            vec![o::FuncCallOutput::InlineReturn(position.clone())]
        } else {
            vec![]
        };

        Ok(o::VPExpression::MacroCall {
            mcro: Box::new(o::VPExpression::Variable(
                self.lookup_identifier(&name_node)?,
                FilePosition::from_pair(&name_node),
            )),
            inputs: input_list,
            outputs: output_list,
            position,
        })
    }

    pub(super) fn convert_vp_var(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vp_var);
        let position = FilePosition::from_pair(&node);
        let child = node.into_inner().next().expect("illegal grammar");
        let var_id = self.lookup_identifier(&child)?;
        Ok(o::VPExpression::Variable(var_id, position))
    }

    pub(super) fn convert_build_array(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::build_array);
        let position = FilePosition::from_pair(&node);
        let mut values = Vec::new();
        for child in node.into_inner() {
            values.push(self.convert_vpe(child)?);
        }
        Ok(o::VPExpression::Collect(values, position))
    }

    pub(super) fn convert_vpe_part_1(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vpe_part_1);
        let child = node.into_inner().next().expect("illegal grammar");
        match child.as_rule() {
            i::Rule::literal => self.convert_literal(child),
            i::Rule::macro_call => self.convert_macro_call(child, true),
            i::Rule::vp_var => self.convert_vp_var(child),
            i::Rule::vpe => self.convert_vpe(child),
            i::Rule::build_array => self.convert_build_array(child),
            _ => unreachable!("illegal grammar"),
        }
    }

    pub(super) fn convert_negate(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::negate);
        let position = FilePosition::from_pair(&node);
        let child = node.into_inner().next().expect("illegal grammar");
        let base = match child.as_rule() {
            i::Rule::vpe_part_1 => self.convert_vpe_part_1(child)?,
            i::Rule::vp_index => self.convert_vp_index(child)?,
            _ => unreachable!("illegal grammar"),
        };
        Ok(o::VPExpression::UnaryOperation(
            o::UnaryOperator::Negate,
            Box::new(base),
            position,
        ))
    }

    pub(super) fn convert_build_array_type(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::build_array_type);
        let position = FilePosition::from_pair(&node);

        let mut dimensions = Vec::new();
        for child in node.into_inner() {
            match child.as_rule() {
                i::Rule::vpe => dimensions.push(self.convert_vpe(child)?),
                i::Rule::vpe_part_1 => {
                    let base = self.convert_vpe_part_1(child)?;
                    return Ok(o::VPExpression::BuildArrayType {
                        dimensions,
                        base: Box::new(base),
                        position,
                    });
                }
                _ => unreachable!("illegal grammar"),
            }
        }
        unreachable!("illegal grammar")
    }

    pub(super) fn convert_vp_index(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vp_index);
        let position = FilePosition::from_pair(&node);
        let mut children = node.into_inner();

        let base_node = children.next().expect("illegal grammar");
        let base = self.convert_vpe_part_1(base_node)?;
        let mut indexes = Vec::new();
        for child in children {
            indexes.push(self.convert_vpe(child)?);
        }

        Ok(o::VPExpression::Index {
            base: Box::new(base),
            indexes,
            position,
        })
    }

    pub(super) fn convert_vpe_part(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vpe_part);
        let child = node.into_inner().next().expect("illegal grammar");
        match child.as_rule() {
            i::Rule::negate => self.convert_negate(child),
            i::Rule::build_array_type => self.convert_build_array_type(child),
            i::Rule::vp_index => self.convert_vp_index(child),
            i::Rule::vpe_part_1 => self.convert_vpe_part_1(child),
            _ => unreachable!("illegal grammar"),
        }
    }

    pub(super) fn convert_vpe(&mut self, node: i::Node) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vpe);
        let mut operator_stack = Vec::with_capacity(64);
        let mut operand_stack = Vec::with_capacity(64);
        operator_stack.push(Operator::Sentinel);

        for child in node.into_inner() {
            match child.as_rule() {
                i::Rule::vpe_part => {
                    let result = self.convert_vpe_part(child)?;
                    operand_stack.push(result);
                }
                i::Rule::operator => {
                    let op_str = child.as_str();
                    let new_operator = op_str_to_operator(op_str);
                    // Shunting yard algorithm.
                    loop {
                        let top_op_prec = operator_stack.last().unwrap().precedence();
                        if new_operator.precedence() > top_op_prec
                            || (new_operator.precedence() == top_op_prec
                                && new_operator.right_associative())
                        {
                            operator_stack.push(new_operator);
                            break;
                        } else {
                            let top_operator = operator_stack.pop().unwrap();
                            let rhs = operand_stack.pop().unwrap();
                            let lhs = operand_stack.pop().unwrap();
                            let mut position = lhs.clone_position();
                            position.include_other(&rhs.clone_position());
                            operand_stack.push(o::VPExpression::BinaryOperation(
                                Box::new(lhs),
                                top_operator.bin_op(),
                                Box::new(rhs),
                                position,
                            ));
                        }
                    }
                }
                _ => unreachable!("illegal grammar"),
            }
        }

        // If we have leftover operators, we need to do some more looping to get rid of them. The
        // shunting yard algorithm used above guarantees that we can just loop through and compose
        // them in order because they are already in the correct order of precedence.
        // We start from 1 not 0 because we don't want to pop the sentinel.
        for _ in 1..operator_stack.len() {
            let top_operator = operator_stack.pop().unwrap();
            let rhs = operand_stack.pop().unwrap();
            let lhs = operand_stack.pop().unwrap();
            let mut position = lhs.clone_position();
            position.include_other(&rhs.clone_position());
            operand_stack.push(o::VPExpression::BinaryOperation(
                Box::new(lhs),
                top_operator.bin_op(),
                Box::new(rhs),
                position,
            ));
        }

        Ok(operand_stack.pop().unwrap())
    }
}
