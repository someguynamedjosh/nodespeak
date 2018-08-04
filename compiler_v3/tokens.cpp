#include <cmath>
#include <iostream>
#include <vector>

#include "tokens.h"
#include "scope.h"

using namespace Com;

Value *IdentifierExp::getValue(Scope *scope) {
	return scope->lookupVar(name);
}

Value *IntExp::getValue(Scope *scope) {
	return new Value(DATA_TYPE_INT, new int(value));
}

Value *FloatExp::getValue(Scope *scope) {
	return new Value(DATA_TYPE_FLOAT, new float(value));
}

Value *OperatorExp::getValue(Scope *scope) {
	FuncScope *opFunc = getComFunc();
	Command *c = new Command(opFunc);
	for (Expression *ex : args) {
		c->addInput(ex->getValue(scope));
	}
	
	Value *tvar = new Value(opFunc->getOuts()[0]->getType());
	scope->declareTempVar(tvar);
	c->addOutput(tvar);
	scope->addCommand(c);
	return tvar;
}
/*
Value *convert(Scope *scope, Value *value, DataType *to, Value *dest = nullptr) {
	Command *cc;
	Value *tvar;
	DataType *itype = value->getType();
	if (to == DATA_TYPE_FLOAT) {
		tvar = new Value(DATA_TYPE_FLOAT);
		if (itype == DATA_TYPE_INT) {
			cc = new Command(BUILTIN_ITOF);
		} else if (itype == DATA_TYPE_BOOL) {
			cc = new Command(BUILTIN_BTOF);
		}
	} else if (to == DATA_TYPE_INT) {
		tvar = new Value(DATA_TYPE_INT);
		if (itype == DATA_TYPE_FLOAT) {
			cc = new Command(BUILTIN_FTOI);
		} else if (itype == DATA_TYPE_BOOL) {
			cc = new Command(BUILTIN_BTOI);
		}
	} else if (to == DATA_TYPE_BOOL) {
		tvar = new Value(DATA_TYPE_BOOL);
		if (itype == DATA_TYPE_FLOAT) {
			cc = new Command(BUILTIN_FTOB);
		} else if (itype == DATA_TYPE_INT) {
			cc = new Command(BUILTIN_ITOB);
		}
	}
	scope->declareTempVar(tvar);
	cc->addInput(value);
	if (dest) {
		cc->addOutput(dest);
	} else {
		cc->addOutput(tvar);
	}
	return new Value(tvar);
}
*/

Value *FuncCall::getValue(Scope *scope) {
	FuncScope *func = scope->lookupFunc(name);
	Command *fc = new Command(func);
	std::vector<Value*> fins = func->getIns();
	if (ins->getExps().size() != fins.size()) return nullptr;
	for (int i = 0; i < ins->getExps().size(); i++) {
		Value *rval = ins->getExps()[i]->getValue(scope);
		DataType *ftype = fins[i]->getType();
		fc->addInput(rval);
	}
	std::vector<Value*> fouts = func->getOuts();
	if (outs->getOuts().size() != fouts.size()) return nullptr;
	Value *toReturn;
	for (int i = 0; i < outs->getOuts().size(); i++) {
		Output *rval = outs->getOuts()[i];
		switch (rval->getType()) {
		case RetOut::TYPE_CONST:
			toReturn = new Value(fouts[i]->getType());
			scope->declareTempVar(toReturn);
			fc->addOutput(toReturn);
			break;
		case NoneOut::TYPE_CONST:
			fc->addOutput(nullptr);
			break;
		case VarAccessOut::TYPE_CONST:
			if (IdentifierExp *sexp = dynamic_cast<IdentifierExp*>(rval->getExp())) {
				Value *target = scope->lookupVar(sexp->getName());
				fc->addOutput(target);
			}
			break;
		}
	}
	scope->addCommand(fc);
	return (toReturn) ? toReturn : nullptr;
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
		Command *c;
		Value *right = value->getValue(scope);
		Value *left = scope->lookupVar(sexp->getName());
		c = new Command(BUILTIN_COPY);
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
	Value *sizec = size->getValue(scope);
	if (sizec->isConstant()) {
		int sizei = 0;
		if (sizec->getType() == DATA_TYPE_FLOAT) {
			sizei = floor(*sizec->interpretAsFloat());
		} else if (sizec->getType() == DATA_TYPE_INT) {
			sizei = *sizec->interpretAsInt();
		}
		return new ArrayDataType(baseType->convert(scope), sizei);
	}
	return nullptr; // TODO: Error message for dynamic array sizes.
}

void VarDec::convert(Scope *scope) {
	scope->declareVar(name, new Value(type->convert(scope)));
}

Value *Range::getValue(Scope *scope) {
	Value *starti = start->getValue(scope), *endi = end->getValue(scope);
	Value *stepi = (step) ? step->getValue(scope) : new Value(DATA_TYPE_INT, new int(1));
	if (starti->isConstant() && endi->isConstant() && (!stepi || stepi->isConstant())) {
		DataType *type = DATA_TYPE_INT;
		if (starti->getType() == DATA_TYPE_FLOAT 
		    || endi->getType() == DATA_TYPE_FLOAT
			|| stepi->getType() == DATA_TYPE_FLOAT)
			type = DATA_TYPE_FLOAT;
		if (type == DATA_TYPE_INT) {
			int startv = 0, endv = 0, stepv = 1;
			if (starti->getType() == DATA_TYPE_INT)
				startv = *starti->interpretAsInt();
			else if (starti->getType() == DATA_TYPE_FLOAT)
				startv = int(*starti->interpretAsFloat());
			if (endi->getType() == DATA_TYPE_INT)
				endv = *endi->interpretAsInt();
			else if (endi->getType() == DATA_TYPE_FLOAT)
				endv = int(*endi->interpretAsFloat());
			if (stepi->getType() == DATA_TYPE_INT)
				stepv = *stepi->interpretAsInt();
			else if (stepi->getType() == DATA_TYPE_FLOAT)
				stepv = int(*stepi->interpretAsFloat());
			int size = (endv - startv + stepv - 1) / stepv;
			int *data = new int[size];
			for (int i = 0, v = startv; i < size; i++, v += stepv)
				data[i] = v;
			type = new ArrayDataType(DATA_TYPE_INT, size);
			return new Value(type, (void*) data);
		} else if (type == DATA_TYPE_FLOAT) {
			float startv = 0, endv = 0, stepv = 1;
			if (starti->getType() == DATA_TYPE_FLOAT)
				startv = *starti->interpretAsFloat();
			else if (starti->getType() == DATA_TYPE_INT)
				startv = float(*starti->interpretAsInt());
			if (endi->getType() == DATA_TYPE_FLOAT)
				endv = *endi->interpretAsFloat();
			else if (endi->getType() == DATA_TYPE_INT)
				endv = float(*endi->interpretAsInt());
			if (stepi->getType() == DATA_TYPE_FLOAT)
				stepv = *stepi->interpretAsFloat();
			else if (stepi->getType() == DATA_TYPE_INT)
				stepv = float(*stepi->interpretAsInt());
			int size = int(ceil((endv - startv) / stepv));
			float *data = new float[size];
			int i; float v;
			for (i = 0, v = startv; i < size; i++, v += stepv)
				data[i] = v;
			type = new ArrayDataType(DATA_TYPE_FLOAT, size);
			return new Value(type, (void*) data);
		}
	}
	// TODO: Implement non-constant ranges.
	return nullptr;
}

void addLoopCall(FuncScope *loopScope, Value *counterInput) {
	Command *com = new Command(loopScope);
	com->addInput(counterInput);
	loopScope->getParent()->addCommand(com);
}

void ForLoop::convert(Scope *scope) {
	FuncScope *s = new FuncScope(scope);
	scope->declareTempFunc(s);
	Value *counterv = new Value(counter->getType()->convert(scope));
	s->declareVar(counter->getName(), counterv);
	s->addIn(counterv);
	for (Statement *stat : body->getStatements()) {
		stat->convert(s);
	}
	for (Expression *value : values->getExps()) {
		Value *cval = value->getValue(scope);
		if (cval->isConstant()) {
			if (cval->getType()->getArrayDepth() > 0) {
				ArrayDataType *atype = dynamic_cast<ArrayDataType*>(cval->getType());
				for (int i = 0; i < atype->getArrayLength(); i++) {
					Command *iter = new Command(s);
					char *addr = ((char*) cval->getData());
					addr += i * atype->getBaseType()->getLength();
					Value *cinput = new Value(atype->getBaseType(), (void*) addr);
					addLoopCall(s, cinput);
				}
			} else {
				addLoopCall(s, cval);
			}
		} else {
			if (cval->getType()->getArrayDepth() > 0) {
				ArrayDataType *atype = dynamic_cast<ArrayDataType*>(cval->getType());
				for (int i = 0; i < atype->getArrayLength(); i++) {
					Command *iter = new Command(s);
					Value *temp = new Value(atype->getBaseType());
					Command *cop = new Command(BUILTIN_INDEX);
					cop->addInput(cval);
					cop->addInput(new Value(DATA_TYPE_INT, new int(i)));
					cop->addOutput(temp);
					scope->declareTempVar(temp);
					scope->addCommand(cop);
					addLoopCall(s, temp);
				}
			} else {
				addLoopCall(s, cval);
			}
		}
	}
}




