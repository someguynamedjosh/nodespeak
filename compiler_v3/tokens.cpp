#include <cmath>
#include <iostream>
#include <vector>

#include "tokens.h"
#include "scope.h"

using namespace Com;

FuncInput *IdentifierExp::getValue(Scope *scope) {
	return new FuncInput(scope->lookupVar(name));
}

FuncInput *IntExp::getValue(Scope *scope) {
	return new FuncInput(new Literal(DATA_TYPE_INT, new int(value)));
}

FuncInput *FloatExp::getValue(Scope *scope) {
	return new FuncInput(new Literal(DATA_TYPE_FLOAT, new float(value)));
}

FuncInput *OperatorExp::getValue(Scope *scope) {
	std::vector<FuncInput*> cargs;
	int dtypeIndex = -1;
	bool allLiterals = true;
	for (Expression *ex : args) {
		FuncInput *val = ex->getValue(scope);
		cargs.push_back(val);
		allLiterals &= val->isLiteral();
		DataType *type = (val->isVariable()) 
			? val->getVariable()->getType()
			: val->getLiteral()->getType();
		if (type == DATA_TYPE_FLOAT && dtypeIndex < 2) {
			dtypeIndex = 2;
		} else if (type == DATA_TYPE_INT && dtypeIndex < 1) {
			dtypeIndex = 1;
		} else if (type == DATA_TYPE_BOOL && dtypeIndex < 0) {
			dtypeIndex = 0;
		}
	}
	DataType *expType;
	switch(dtypeIndex) {
		case 2:
			expType = DATA_TYPE_FLOAT;
			break;
		case 1:
			expType = DATA_TYPE_INT;
			break;
		case 0:
			expType = DATA_TYPE_BOOL;
			break;
	}
	if(allLiterals) { // Compute a raw value rather than computing it at runtime.
		Literal *currentValue = cargs[0]->getLiteral();
		for (int i = 1; i < cargs.size(); i++) {
			currentValue = evalBuiltinFunc(getComFunc(), currentValue, cargs[i]->getLiteral());
		}
		return new FuncInput(currentValue);
	} else {
		FuncCommand *c = new FuncCommand(getComFunc());
		for (FuncInput *a : cargs) {
			DataType *type = (a->isVariable()) 
				? a->getVariable()->getType()
				: a->getLiteral()->getType();
			if (type == expType) {
				c->addInput(a);
			} else {
				Variable *tvar = new Variable(expType);
				scope->declareTempVar(tvar);
				FuncCommand *com;
				if (expType == DATA_TYPE_FLOAT && type == DATA_TYPE_INT) {
					com = new FuncCommand(BUILTIN_ITOF);
				} else if (expType == DATA_TYPE_FLOAT && type == DATA_TYPE_BOOL) {
					com = new FuncCommand(BUILTIN_BTOF);
				} else if (expType == DATA_TYPE_INT && type == DATA_TYPE_BOOL) {
					com = new FuncCommand(BUILTIN_BTOI);
				}
				com->addInput(a);
				com->addOutput(tvar);
				scope->addCommand(com);
				c->addInput(new FuncInput(tvar));
			}
		}
		
		Variable *tvar = new Variable(expType);
		scope->declareTempVar(tvar);
		c->addOutput(tvar);
		scope->addCommand(c);
		return new FuncInput(tvar);
	}
}

FuncInput *convert(Scope *scope, FuncInput *value, DataType *to, Variable *dest = nullptr) {
	FuncCommand *cc;
	Variable *tvar;
	DataType *itype = value->getType();
	if (to == DATA_TYPE_FLOAT) {
		tvar = new Variable(DATA_TYPE_FLOAT);
		if (itype == DATA_TYPE_INT) {
			cc = new FuncCommand(BUILTIN_ITOF);
		} else if (itype == DATA_TYPE_BOOL) {
			cc = new FuncCommand(BUILTIN_BTOF);
		}
	} else if (to == DATA_TYPE_INT) {
		tvar = new Variable(DATA_TYPE_INT);
		if (itype == DATA_TYPE_FLOAT) {
			cc = new FuncCommand(BUILTIN_FTOI);
		} else if (itype == DATA_TYPE_BOOL) {
			cc = new FuncCommand(BUILTIN_BTOI);
		}
	} else if (to == DATA_TYPE_BOOL) {
		tvar = new Variable(DATA_TYPE_BOOL);
		if (itype == DATA_TYPE_FLOAT) {
			cc = new FuncCommand(BUILTIN_FTOB);
		} else if (itype == DATA_TYPE_INT) {
			cc = new FuncCommand(BUILTIN_ITOB);
		}
	}
	scope->declareTempVar(tvar);
	cc->addInput(value);
	if (dest) {
		cc->addOutput(dest);
	} else {
		cc->addOutput(tvar);
	}
	return new FuncInput(tvar);
}

FuncInput *FuncCall::getValue(Scope *scope) {
	FuncScope *func = scope->lookupFunc(name);
	FuncCommand *fc = new FuncCommand(func);
	std::vector<Variable*> fins = func->getIns();
	if (ins->getExps().size() != fins.size()) return nullptr;
	for (int i = 0; i < ins->getExps().size(); i++) {
		FuncInput *rval = ins->getExps()[i]->getValue(scope);
		DataType *ftype = fins[i]->getType();
		if (rval->getType() == ftype) {
			fc->addInput(rval);
		} else {
			fc->addInput(convert(scope, rval, ftype));
		}
	}
	std::vector<Variable*> fouts = func->getOuts();
	if (outs->getOuts().size() != fouts.size()) return nullptr;
	Variable *toReturn;
	for (int i = 0; i < outs->getOuts().size(); i++) {
		Output *rval = outs->getOuts()[i];
		switch (rval->getType()) {
		case RetOut::TYPE_CONST:
			toReturn = new Variable(fouts[i]->getType());
			scope->declareTempVar(toReturn);
			fc->addOutput(toReturn);
			break;
		case NoneOut::TYPE_CONST:
			fc->addOutput(nullptr);
			break;
		case VarAccessOut::TYPE_CONST:
			if (IdentifierExp *sexp = dynamic_cast<IdentifierExp*>(rval->getExp())) {
				Variable *target = scope->lookupVar(sexp->getName());
				if (target->getType() == fouts[i]->getType()) {
					fc->addOutput(target);
				} else {
					Variable *temp = new Variable(fouts[i]->getType());
					scope->declareTempVar(temp);
					fc->addOutput(temp);
					convert(scope, new FuncInput(temp), target->getType(), target);
				}
			}
			break;
		}
	}
	scope->addCommand(fc);
	return (toReturn) ? new FuncInput(toReturn) : nullptr;
}

FuncScope *AddExp::getComFunc() { return BUILTIN_ADD; }
FuncScope *MulExp::getComFunc() { return BUILTIN_MUL; }
FuncScope *RecipExp::getComFunc() { return BUILTIN_RECIP; }
FuncScope *ModExp::getComFunc() { return BUILTIN_MOD; }
FuncScope *EqExp::getComFunc() { return BUILTIN_EQ; }
FuncScope *NeqExp::getComFunc() { return BUILTIN_NEQ; }
FuncScope *LteExp::getComFunc() { return BUILTIN_LTE; }
FuncScope *GteExp::getComFunc() { return BUILTIN_GTE; }
FuncScope *LtExp::getComFunc() { return BUILTIN_LT; }
FuncScope *GtExp::getComFunc() { return BUILTIN_GT; }
FuncScope *AndExp::getComFunc() { return BUILTIN_AND; }
FuncScope *OrExp::getComFunc() { return BUILTIN_OR; }
FuncScope *XorExp::getComFunc() { return BUILTIN_XOR; }
FuncScope *BandExp::getComFunc() { return BUILTIN_BAND; }
FuncScope *BorExp::getComFunc() { return BUILTIN_BOR; }
FuncScope *BxorExp::getComFunc() { return BUILTIN_BXOR; }

void AssignStat::convert(Scope *scope) {
	if (IdentifierExp* sexp = dynamic_cast<IdentifierExp*>(to)) {
		FuncCommand *c;
		FuncInput *right = value->getValue(scope);
		DataType *rtype = (right->isVariable())
			? right->getVariable()->getType()
			: right->getLiteral()->getType();
		Variable *left = scope->lookupVar(sexp->getName());
		DataType *ltype = left->getType();
		if (ltype == rtype) {
			c = new FuncCommand(BUILTIN_COPY);
		} else if (ltype == DATA_TYPE_FLOAT && rtype == DATA_TYPE_INT) {
			c = new FuncCommand(BUILTIN_ITOF);
		} else if (ltype == DATA_TYPE_FLOAT && rtype == DATA_TYPE_BOOL) {
			c = new FuncCommand(BUILTIN_BTOF);
		} else if (ltype == DATA_TYPE_INT && rtype == DATA_TYPE_BOOL) {
			c = new FuncCommand(BUILTIN_BTOI);
		}
		c->addInput(right);
		c->addOutput(left);
		scope->addCommand(c);
	}
}

void FuncDec::convert(Scope *scope) {
	FuncScope *s = new FuncScope(scope);
	s->autoAddIns();
	for (Statement *inst : ins->getStatements()) {
		inst->convert(s);
	}
	s->autoAddOuts();
	for (Statement *outst : outs->getStatements()) {
		outst->convert(s);
	}
	s->autoAddNone();
	scope->declareFunc(name, s);
}

DataType *TypeName::convert(Scope *scope) {
	return scope->lookupType(name);
}

DataType *ArrayType::convert(Scope *scope) {
	FuncInput *sizec = size->getValue(scope);
	if (sizec->isLiteral()) {
		Literal *sizel = sizec->getLiteral();
		int sizei = 0;
		if (sizel->getType() == DATA_TYPE_FLOAT) {
			sizei = floor(*sizel->interpretAsFloat());
		} else if (sizel->getType() == DATA_TYPE_INT) {
			sizei = *sizel->interpretAsInt();
		}
		return new ArrayDataType(baseType->convert(scope), sizei);
	}
	return nullptr; // TODO: Error message for dynamic array sizes.
}

void VarDec::convert(Scope *scope) {
	scope->declareVar(name, new Variable(type->convert(scope)));
}

FuncInput *Range::getValue(Scope *scope) {
	FuncInput *starti = start->getValue(scope), *endi = end->getValue(scope);
	FuncInput *stepi = (step) ? step->getValue(scope) : nullptr;
	if (starti->isLiteral() && endi->isLiteral() && (!stepi || stepi->isLiteral())) {
		Literal *startl = starti->getLiteral(), *endl_ = endi->getLiteral();
		Literal *stepl = (stepi) ? stepi->getLiteral() 
		                         : new Literal(DATA_TYPE_INT, new int(1));
		DataType *type = DATA_TYPE_INT;
		if (startl->getType() == DATA_TYPE_FLOAT 
		    || endl_->getType() == DATA_TYPE_FLOAT
			|| stepl->getType() == DATA_TYPE_FLOAT)
			type = DATA_TYPE_FLOAT;
		if (type == DATA_TYPE_INT) {
			int startv = 0, endv = 0, stepv = 1;
			if (startl->getType() == DATA_TYPE_INT)
				startv = *startl->interpretAsInt();
			else if (startl->getType() == DATA_TYPE_FLOAT)
				startv = int(*startl->interpretAsFloat());
			if (endl_->getType() == DATA_TYPE_INT)
				endv = *endl_->interpretAsInt();
			else if (endl_->getType() == DATA_TYPE_FLOAT)
				endv = int(*endl_->interpretAsFloat());
			if (stepl->getType() == DATA_TYPE_INT)
				stepv = *stepl->interpretAsInt();
			else if (stepl->getType() == DATA_TYPE_FLOAT)
				stepv = int(*stepl->interpretAsFloat());
			int size = (endv - startv + stepv - 1) / stepv;
			int *data = new int[size];
			for (int i = 0, v = startv; i < size; i++, v += stepv)
				data[i] = v;
			type = new ArrayDataType(DATA_TYPE_INT, size);
			return new FuncInput(new Literal(type, (void*) data));
		} else if (type == DATA_TYPE_FLOAT) {
			float startv = 0, endv = 0, stepv = 1;
			if (startl->getType() == DATA_TYPE_FLOAT)
				startv = *startl->interpretAsFloat();
			else if (startl->getType() == DATA_TYPE_INT)
				startv = float(*startl->interpretAsInt());
			if (endl_->getType() == DATA_TYPE_FLOAT)
				endv = *endl_->interpretAsFloat();
			else if (endl_->getType() == DATA_TYPE_INT)
				endv = float(*endl_->interpretAsInt());
			if (stepl->getType() == DATA_TYPE_FLOAT)
				stepv = *stepl->interpretAsFloat();
			else if (stepl->getType() == DATA_TYPE_INT)
				stepv = float(*stepl->interpretAsInt());
			int size = int(ceil((endv - startv) / stepv));
			float *data = new float[size];
			int i; float v;
			for (i = 0, v = startv; i < size; i++, v += stepv)
				data[i] = v;
			type = new ArrayDataType(DATA_TYPE_FLOAT, size);
			return new FuncInput(new Literal(type, (void*) data));
		}
	}
	// TODO: Implement non-constant ranges.
	return nullptr;
}

void addLoopCall(FuncScope *loopScope, FuncInput *counterInput) {
	FuncCommand *com = new FuncCommand(loopScope);
	com->addInput(counterInput);
	loopScope->getParent()->addCommand(com);
}

void ForLoop::convert(Scope *scope) {
	FuncScope *s = new FuncScope(scope);
	scope->declareTempFunc(s);
	Variable *counterv = new Variable(counter->getType()->convert(scope));
	s->declareVar(counter->getName(), counterv);
	s->addIn(counterv);
	for (Statement *stat : body->getStatements()) {
		stat->convert(s);
	}
	for (Expression *value : values->getExps()) {
		FuncInput *cval = value->getValue(scope);
		FuncInput *cinput;
		if (cval->isLiteral()) {
			Literal *lval = cval->getLiteral();
			if (lval->getType()->getArrayDepth() > 0) {
				ArrayDataType *atype = dynamic_cast<ArrayDataType*>(lval->getType());
				for (int i = 0; i < atype->getArrayLength(); i++) {
					FuncCommand *iter = new FuncCommand(s);
					char *addr = ((char*) lval->getData());
					addr += i * atype->getBaseType()->getLength();
					cinput = new FuncInput(new Literal(atype->getBaseType(), (void*) addr));
					addLoopCall(s, cinput);
				}
			} else {
				addLoopCall(s, new FuncInput(lval));
			}
		} else {
			Variable *vval = cval->getVariable();
			if (vval->getType()->getArrayDepth() > 0) {
				ArrayDataType *atype = dynamic_cast<ArrayDataType*>(vval->getType());
				for (int i = 0; i < atype->getArrayLength(); i++) {
					FuncCommand *iter = new FuncCommand(s);
					Variable *temp = new Variable(atype->getBaseType());
					FuncCommand *cop = new FuncCommand(BUILTIN_INDEX);
					cop->addInput(new FuncInput(vval));
					cop->addInput(new FuncInput(new Literal(DATA_TYPE_INT, new int(i))));
					cop->addOutput(temp);
					scope->declareTempVar(temp);
					scope->addCommand(cop);
					addLoopCall(s, new FuncInput(temp));
				}
			} else {
				addLoopCall(s, new FuncInput(vval));
			}
		}
	}
}




