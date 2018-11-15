#include "grammar/Statements.h"

#include "grammar/DataTypes.h"
#include "grammar/Expressions.h"

namespace waveguide {
namespace convert {

void grammar::FuncCallStat::convert(ScopeSP context) {
    call->getValue(context);
}

void grammar::VarDec::convert(ScopeSP context) {
    DTypeSP dtype{type->convert(context)};
    ValueSP value{new Value{dtype}};
    context->declareVar(name, value);
}

void grammar::AssignStat::convert(ScopeSP context) {
    to->setFromValue(context, value->getValue(context));
}

void grammar::ReturnStat::convert(ScopeSP context) { }

void grammar::FuncDec::convert(ScopeSP context) {
    ScopeSP func{new Scope{context}};
    func->autoAddIns();
    for (auto inst : ins->getStatements()) {
        inst->convert(func);
    }
    func->autoAddOuts();
    for (auto outst : outs->getStatements()) {
        outst->convert(func);
    }
    func->autoAddNone();
    context->declareFunc(name, func);
    // Do not convert the body. Body conversion will be done later, manually, by
    // the master conversion function.
}

void grammar::Branch::convert(ScopeSP context) {
    ValueSP coni{con->getValue(context)};
    ScopeSP ifTrueScope{new Scope{context}};
    context->declareTempFunc(ifTrueScope);
    AugmentationSP ifTrueAug{new Augmentation{
        intermediate::AugmentationType::DO_IF, coni
    }};
    ifTrue->convert(ifTrueScope);
    context->addCommand(CommandSP{new Command{ifTrueScope, ifTrueAug}});
    
    if (ifFalse) {
        ScopeSP ifFalseScope{new Scope{context}};
        context->declareTempFunc(ifFalseScope);
        ifFalse->convert(ifFalseScope);
        AugmentationSP ifFalseAug{new Augmentation{
            intermediate::AugmentationType::DO_IF_NOT, coni
        }};
        context->addCommand(CommandSP{new Command{ifFalseScope, ifFalseAug}});
    }
}

void addLoopCall(ScopeSP loopScope, ValueSP counterInput) {
	CommandSP com{new Command{loopScope}};
	com->addInput(counterInput);
	loopScope->getParent()->addCommand(com);
}

void grammar::ForLoop::convert(ScopeSP context) {
    ScopeSP bodyScope{new Scope{context}};
    context->declareTempFunc(bodyScope);
    ValueSP counterv{new Value{counter->getType()->convert(context)}};
    bodyScope->declareVar(counter->getName(), counterv);
    bodyScope->addIn(counterv);
    for (auto stat : body->getStatements()) {
        stat->convert(bodyScope);
    }
    for (auto expr : values->getExps()) {
        ValueSP value{expr->getValue(context)};
        if (value->isValueKnown()) {
            if (auto atype = std::dynamic_pointer_cast
                <intermediate::ArrayDataType>(value->getType())) {
                for (int i = 0; i < atype->getArrayLength(); i++) {
                    CommandSP iter{new Command{bodyScope}};
                    char *addr = static_cast<char*>(value->getData());
                    addr += i * atype->getElementType()->getLength();
                    ValueSP element{
                        new Value(atype->getElementType(), (void*) addr)};
                    addLoopCall(bodyScope, element);
                }
            } else {
                addLoopCall(bodyScope, value);
            }
        } else {
            if (auto atype = std::dynamic_pointer_cast
                <intermediate::ArrayDataType>(value->getType())) {
                for (int i = 0; i < atype->getArrayLength(); i++) {
                    CommandSP iter{new Command{bodyScope}};
                    ValueSP temp{new Value{atype->getElementType()}};
                    CommandSP copy{new Command{blt()->COPY}};
                    copy->addInput(value);
                    copy->addInput(ValueSP{new Value{ blt()->INT, 
                        new int{i * atype->getElementType->getLength()}}});
                    copy->addOutput(temp);
                    context->declareTempVar(temp);
                    context->addCommand(copy);
                    addLoopCall(bodyScope, temp);
                }
            } else {
                addLoopCall(bodyScope, value);
            }
        }
    }
}

void grammar::WhileLoop::convert(ScopeSP context) {
    // TODO: Implement.
}

}
}