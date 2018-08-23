#include "interpreter.h"
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
		outputs.push_back(new Value(fout->getType(), malloc(fout->getType()->getLength())));
	}
	runFuncScope(main, inputs, outputs);
	for (auto out : outputs) {
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
			if (*trigger->interpretAsBool()) runFuncScope(command->getFuncScope(), command->getIns(), command->getOuts());
			break;
		case AugmentationType::DO_IF_NOT:
			trigger = command->getAugmentation()->getParams()[0];
			if (!*trigger->interpretAsBool()) runFuncScope(command->getFuncScope(), command->getIns(), command->getOuts());
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
	} else if (fs == BUILTIN_COPY) {
		memcpy(outputs[0]->getData(), inputs[0]->getData(), inputs[0]->getType()->getLength());
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
