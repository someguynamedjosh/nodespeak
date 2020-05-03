use super::{problems, util, DataType, ResolvedVCExpression, ResolvedVPExpression, ScopeResolver};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;
use std::borrow::Borrow;

impl<'a> ScopeResolver<'a> {
    fn resolve_vc_variable(
        &mut self,
        var_id: i::VariableId,
        position: &FilePosition,
    ) -> Result<ResolvedVCExpression, CompileProblem> {
        let (_, var_type) = self
            .get_var_info(var_id)
            .expect("Variable used before declaration, vague step should have caught this.");
        Ok(ResolvedVCExpression::Specific {
            var: var_id,
            indexes: Vec::new(),
            typ: var_type.clone(),
            pos: position.clone(),
        })
    }

    fn resolve_vc_index(
        &mut self,
        base: &i::VCExpression,
        indexes: &Vec<i::VPExpression>,
        position: &FilePosition,
    ) -> Result<ResolvedVCExpression, CompileProblem> {
        let rbase = self.resolve_vc_expression(base)?;

        let mut known_indexes = Vec::new();
        let mut all_indexes = Vec::new();
        let mut etype = rbase.borrow_data_type();
        for index in indexes {
            let arr_len = if let DataType::Array(len, eetype) = etype {
                etype = eetype;
                *len
            } else {
                panic!("TODO: Nice error, cannot index non-array type.");
            };
            let rindex = self.resolve_vp_expression(index)?;
            if rindex.borrow_data_type() != &DataType::Int {
                panic!("TODO: Nice error, index must be int.");
            }
            if let ResolvedVPExpression::Interpreted(data, ..) = &rindex {
                let val = data.require_int(); // We already checked that it should be an int.
                if val < 0 {
                    panic!("TODO: Nice error, array index less than zero.");
                }
                let val = val as usize;
                if val >= arr_len {
                    panic!("TODO: Nice error, array index too big.");
                }
                // If they are unequal, that means at some point we didn't know what one of the
                // earlier indexes was, so we should not add on any more known indexes because it's
                // not really useful in this phase. LLVM will still be able to do optimizations on
                // the literal values that will take their place.
                if known_indexes.len() == all_indexes.len() {
                    known_indexes.push(val);
                }
            }
            all_indexes.push(rindex.as_vp_expression()?);
        }

        let etype = etype.clone();
        Ok(match rbase {
            ResolvedVCExpression::Modified {
                mut vce,
                base,
                indexes,
                ..
            } => {
                // We can't add on our known indices. If all the previous indices were known, we
                // would have gotten a Specific result. Instead, since it is Modified, we cannot
                // add on our indexes to the end because the previous set of indexes is not
                // complete.
                vce.indexes.append(&mut all_indexes);
                ResolvedVCExpression::Modified {
                    vce,
                    typ: etype,
                    base,
                    indexes,
                }
            }
            ResolvedVCExpression::Specific {
                var,
                mut indexes,
                pos,
                ..
            } => {
                let unknown_indexes = &all_indexes[known_indexes.len()..];
                indexes.append(&mut known_indexes);
                if unknown_indexes.len() == 0 {
                    ResolvedVCExpression::Specific {
                        var,
                        indexes,
                        pos,
                        typ: etype,
                    }
                } else {
                    let mut all_indexes = Vec::new();
                    for literal_index in indexes {
                        all_indexes.push(o::VPExpression::Literal(
                            o::KnownData::Int(literal_index as i64),
                            FilePosition::placeholder(),
                        ));
                    }
                    unimplemented!()
                }
            }
        })
    }

    pub(super) fn resolve_vc_expression(
        &mut self,
        input: &i::VCExpression,
    ) -> Result<ResolvedVCExpression, CompileProblem> {
        match input {
            i::VCExpression::Variable(id, position) => self.resolve_vc_variable(*id, position),
            i::VCExpression::Index {
                base,
                indexes,
                position,
            } => self.resolve_vc_index(base, indexes, position),
        }
    }
}
