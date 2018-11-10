#include "interpreter.h"
#include <cmath>
#include <cstring>
#include <iostream>

namespace Com {

void interpret(Scope *root) {
	std::cout << "asdf" << std::endl;
	std::cout << root->repr() << std::endl;
	for (Command* command : root->getCommands()) {
		// Do all variable initialization and stuff contained in the code.
		runFuncScope(command->getFuncScope(), command->getIns(), command->getOuts());
	}
	// Run the main function.
	std::vector<Value*> inputs, outputs;
	FuncScope *main = root->lookupFunc("main");
	for (Value* fin : main->getIns()) {
		inputs.push_back(new Value(fin->getType()));
	}
	for (Value* fout : main->getOuts()) {
		outputs.push_back(new Value(fout->getType()));
	}
	runFuncScope(main, inputs, outputs);
	for (auto out : outputs) {
		out->setConstant(true);
		std::cout << out->repr() << std::endl;
	}
	std::cout << "Interpretation complete!" << std::endl;
}

void forkCommand(Command *command) {
	if (command->getAugmentation() == nullptr) {
		runFuncScope(command->getFuncScope(), command->getIns(), command->getOuts());
	} else {
		Value *trigger;
		switch (command->getAugmentation()->getType()) {
		case AugmentationType::DO_IF:
			trigger = command->getAugmentation()->getParams()[0];
			if (*trigger->interpretAsBool()) 
				runFuncScope(command->getFuncScope(), command->getIns(), command->getOuts());
			break;
		case AugmentationType::DO_IF_NOT:
			trigger = command->getAugmentation()->getParams()[0];
			if (!*trigger->interpretAsBool()) 
				runFuncScope(command->getFuncScope(), command->getIns(), command->getOuts());
			break;
		}
	}
}

void runFuncScope(FuncScope *fs, std::vector<Value*> inputs, std::vector<Value*> outputs) {
	if (fs == BUILTIN_ADD) {
		if (inputs[0]->getType() == DATA_TYPE_INT)
			*outputs[0]->interpretAsInt() 
				= *inputs[0]->interpretAsInt() + *inputs[1]->interpretAsInt();
		else if (inputs[0]->getType() == DATA_TYPE_FLOAT) 
			*outputs[0]->interpretAsFloat() 
				= *inputs[0]->interpretAsFloat() + *inputs[1]->interpretAsFloat();
		else if (inputs[0]->getType() == DATA_TYPE_BOOL) 
			*outputs[0]->interpretAsBool() 
				= *inputs[0]->interpretAsBool() ^ *inputs[1]->interpretAsBool();
	} else if (fs == BUILTIN_MUL) {
		if (inputs[0]->getType() == DATA_TYPE_INT)
			*outputs[0]->interpretAsInt() 
				= *inputs[0]->interpretAsInt() * *inputs[1]->interpretAsInt();
		else if (inputs[0]->getType() == DATA_TYPE_FLOAT) 
			*outputs[0]->interpretAsFloat() 
				= *inputs[0]->interpretAsFloat() * *inputs[1]->interpretAsFloat();
		else if (inputs[0]->getType() == DATA_TYPE_BOOL) 
			*outputs[0]->interpretAsBool() 
				= *inputs[0]->interpretAsBool() & *inputs[1]->interpretAsBool();
	} else if (fs == BUILTIN_RECIP) {
		*outputs[0]->interpretAsFloat() = 1.0f / *inputs[0]->interpretAsFloat();
	} else if (fs == BUILTIN_ITOF) {
		*outputs[0]->interpretAsFloat() = *inputs[0]->interpretAsInt();
	} else if (fs == BUILTIN_BTOF) {
		*outputs[0]->interpretAsFloat() = *inputs[0]->interpretAsBool() ? 1.0f : 0.0f;
	} else if (fs == BUILTIN_BTOI) {
		*outputs[0]->interpretAsInt() = *inputs[0]->interpretAsBool() ? 1 : 0;
	} else if (fs == BUILTIN_ITOB) {
		*outputs[0]->interpretAsBool() = *inputs[0]->interpretAsInt() != 0;
	} else if (fs == BUILTIN_FTOI) {
		*outputs[0]->interpretAsInt() = *inputs[0]->interpretAsFloat();
	} else if (fs == BUILTIN_FTOB) {
		*outputs[0]->interpretAsBool() = *inputs[0]->interpretAsFloat() != 0.0f;
	} else if (fs == BUILTIN_COPY) {
		int offset = *inputs[1]->interpretAsInt();
		if (inputs[0]->getType()->getLength() > outputs[0]->getType()->getLength()) {
			// Copying from a bigger type.
			memcpy(outputs[0]->getData(), (char*) inputs[0]->getData() + offset, outputs[0]->getType()->getLength());
		} else {
			// Copying to a bigger type.
			memcpy((char*) outputs[0]->getData() + offset, inputs[0]->getData(), inputs[0]->getType()->getLength());
		}
	} else if (fs == BUILTIN_LOG) {
		bool wasConstant = inputs[0]->isConstant();
		inputs[0]->setConstant(true);
		std::cout << inputs[0]->repr() << std::endl;
		inputs[0]->setConstant(wasConstant);
	} else if (fs == BUILTIN_MOD) {
		if (inputs[0]->getType() == DATA_TYPE_INT) 
			*outputs[0]->interpretAsInt() = *inputs[0]->interpretAsInt() % *inputs[1]->interpretAsInt();
		else if (inputs[0]->getType() == DATA_TYPE_FLOAT)
			*outputs[0]->interpretAsFloat() = fmod(*inputs[0]->interpretAsFloat(), *inputs[1]->interpretAsFloat());
		else if (inputs[0]->getType() == DATA_TYPE_BOOL)
			*outputs[0]->interpretAsBool() = false; // No matter what, remainder is always 0.
	} else if (fs == BUILTIN_EQ || fs == BUILTIN_NEQ) {
		int dataSize = inputs[0]->getType()->getLength();
		bool result = true;
		char *p0 = (char*) inputs[0]->getData(), *p1 = (char*) inputs[1]->getData();
		for (int i = 0; i < dataSize; i++) {
			if (p0[i] != p1[i]) {
				result = false;
				break;
			}
		}
		if (fs == BUILTIN_NEQ) {
			result = !result; // Negate the result since we are checking for inequality.
		}
		*outputs[0]->interpretAsBool() = result;
	} else if (fs == BUILTIN_LT || fs == BUILTIN_GTE || fs == BUILTIN_GT || fs == BUILTIN_LTE) {
		bool useLessThan = fs == BUILTIN_LT || BUILTIN_GTE;
		bool result;
		if (inputs[0]->getType() == DATA_TYPE_INT) {
			if (useLessThan)
				result = *inputs[0]->interpretAsInt() < *inputs[0]->interpretAsInt();
			else
				result = *inputs[0]->interpretAsInt() > *inputs[0]->interpretAsInt();
		} else if (inputs[0]->getType() == DATA_TYPE_FLOAT) {
			if (useLessThan)
				result = *inputs[0]->interpretAsFloat() < *inputs[0]->interpretAsFloat();
			else
				result = *inputs[0]->interpretAsFloat() > *inputs[0]->interpretAsFloat();
		} else if (inputs[0]->getType() == DATA_TYPE_BOOL) {
			if (useLessThan)
				result = !*inputs[0]->interpretAsBool() && *inputs[0]->interpretAsBool();
			else
				result = *inputs[0]->interpretAsBool() && !*inputs[0]->interpretAsBool();
		}
		bool invert = fs == BUILTIN_GTE || BUILTIN_LTE;
		if (invert) {
			result = !result;
		}
		*outputs[0]->interpretAsBool() = result;
	} else if (fs == BUILTIN_AND || fs == BUILTIN_OR || fs == BUILTIN_XOR) {
		bool i0 = *inputs[0]->interpretAsBool(), i1 = *inputs[1]->interpretAsBool(), out;
		if (fs == BUILTIN_AND) {
			out = i0 && i1;
		} else if (fs == BUILTIN_OR) {
			out = i0 || i1;
		} else if (fs == BUILTIN_XOR) {
			out = i0 ^ i1;
		}
		*outputs[0]->interpretAsBool() = out;
	} else if (fs == BUILTIN_BAND || fs == BUILTIN_BOR || fs == BUILTIN_BXOR) {
		// TODO: scope.cpp says this should be compatible with all types, not just int. Either make it int-only or
		// figure out how it would work with other types.
		int i0 = *inputs[0]->interpretAsInt(), i1 = *inputs[1]->interpretAsInt(), out;
		if (fs == BUILTIN_BAND) {
			out = i0 & i1;
		} else if (fs == BUILTIN_BOR) {
			out = i0 | i1;
		} else if (fs == BUILTIN_BXOR) {
			out = i0 ^ i1;
		}
		*outputs[0]->interpretAsInt() = out;
	} else {
		for (int i = 0; i < fs->getIns().size(); i++) {
			memcpy(fs->getIns()[i]->getData(), inputs[i]->getData(), inputs[i]->getType()->getLength());
		}
		for (int i = 0; i < fs->getOuts().size(); i++) {
			memcpy(fs->getOuts()[i]->getData(), outputs[i]->getData(), outputs[i]->getType()->getLength());
		}
		for (Command *command : fs->getCommands()) {
			forkCommand(command);
		}
		for (int i = 0; i < outputs.size(); i++) {
			memcpy(outputs[i]->getData(), fs->getOuts()[i]->getData(), 
					outputs[i]->getType()->getLength());
		}
	}
}

};
