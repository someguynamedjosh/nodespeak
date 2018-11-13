#include "grammar/Expressions.h"

#include <cmath>

#include "intermediate/Builtins.h"
#include "intermediate/Scope.h"

namespace waveguide {
namespace convert {

ValueSP grammar::IdentifierExp::getValue(ScopeSP context) {
    // TODO: Error for undefined identifier.
    return context->lookupVar(name);
}

ValueSP grammar::IntExp::getValue(ScopeSP context) {
    return ValueSP(new Value(blt()->INT, new int{value}));
}

ValueSP grammar::FloatExp::getValue(ScopeSP context) {
    return ValueSP(new Value(blt()->FLOAT, new float{value}));
}

ValueSP grammar::BoolExp::getValue(ScopeSP context) {
    return ValueSP(new Value(blt()->BOOL, new bool{value}));
}

grammar::AccessExp::AccessResult grammar::AccessExp::getAccessResult(
    ScopeSP context) {
    ValueSP rootVal{rootVar->getValue(context)};
    ValueSP offset{new Value(blt()->INT)};
    if (accessors.size() == 0) {
        *offset->dataAsInt() = 0;
        offset->setValueKnown(true);
    } else {
        CommandSP set{new Command(blt()->COPY)};
        set->addInput(ValueSP{new Value(blt()->INT, new int{0})});
        set->addInput(ValueSP{new Value(blt()->INT, new int{0})});
        set->addOutput(offset);
        context->addCommand(set);
        context->declareTempVar(offset);
    }
    DTypeSP dataType = rootVal->getType();
	// TODO: Optimize this for multiple sucessive array indexing operations
	// TODO: Add in member access operations once objects are a thing
	// TODO: Add errors if array access or member access is used on an unsupported data type.
	for (auto accessor : accessors) {
		if (accessor.index == 0) { // Index access operation.
			DTypeSP elementType = std::static_pointer_cast
                <intermediate::ArrayDataType>(dataType)->getElementType();
            ValueSP index = std::get<std::shared_ptr<Expression>>(accessor)
                ->getValue(context);
            // context->declareTempVar(index);

			CommandSP mul{new Command(blt()->MUL)};
			mul->addInput(index);
			mul->addInput(ValueSP{new Value(
                blt()->INT, new int{elementType->getLength()}
            )});
			ValueSP mindex{new Value(blt()->INT)};
			context->declareTempVar(mindex);
			mul->addOutput(mindex);
			context->addCommand(mul);

			CommandSP add{new Command(blt()->ADD)};
			add->addInput(offset);
			add->addInput(mindex);
			add->addOutput(offset);
			dataType = elementType;
			context->addCommand(add);
		}
	}
	AccessResult tr;
	tr.finalType = dataType;
	tr.rootVal = rootVal;
	tr.offset = offset;
	return tr;
}

ValueSP grammar::AccessExp::getValue(ScopeSP context) {
    auto result = getAccessResult(context);
    CommandSP copy{new Command(blt()->COPY)};
    copy->addInput(result.rootVal);
    copy->addInput(result.offset);
    ValueSP tr{new Value(result.finalType)};
    context->declareTempVar(tr);
    copy->addOutput(tr);
    context->addCommand(copy);
    return tr;
}

void grammar::AccessExp::setFromValue(ScopeSP context, ValueSP copyFrom) {
    auto result = getAccessResult(context);
    CommandSP copy{new Command(blt()->COPY)};
    copy->addInput(copyFrom);
    copy->addInput(result.offset);
    copy->addOutput(result.rootVal);
    context->addCommand(copy);
}

ValueSP grammar::ArrayLiteral::getValue(ScopeSP context) {
	DTypeSP type{nullptr};
	std::vector<ValueSP> values;
	for (auto expr : elements->getExps()) {
		ValueSP value = expr->getValue(context);
		values.push_back(value);
		type = pickBiggerType(type, value->getType());
	}
	ValueSP output{new Value{DTypeSP{new intermediate::ArrayDataType{
        type, elements->getExps().size()
    }}}};
	context->declareTempVar(output);
	int i = 0;
	for (auto value : values) {
		CommandSP c{new Command(blt()->COPY)};
		c->addInput(value);
		c->addInput(ValueSP{new Value{blt()->INT, new int{i}}});
		c->addOutput(output);
		context->addCommand(c);
		i += type->getLength();
	}
	return output;
}

ValueSP grammar::Range::getValue(ScopeSP context) {
    ValueSP starti{start->getValue(context)}, endi{end->getValue(context)};
    ValueSP stepi{step ? step->getValue(context) 
        : ValueSP{new Value{blt()->INT, new int{1}}}};
    // TODO: Implement non-constant ranges.
    if (starti->isValueKnown() && endi->isValueKnown() 
        && stepi->isValueKnown()) {
        // TODO: Error for any data type that is not int or float.
        DTypeSP type{pickBiggerType(starti->getType(), endi->getType())};
        type = pickBiggerType(type, stepi->getType());
        if (std::dynamic_pointer_cast<intermediate::FloatDataType>(type)) {
            float startv{
                std::dynamic_pointer_cast<intermediate::FloatDataType>(starti)
                ? *starti->dataAsFloat() : float(*starti->dataAsInt())
            }, endv{
                std::dynamic_pointer_cast<intermediate::FloatDataType>(endi)
                ? *endi->dataAsFloat() : float(*endi->dataAsInt())
            }, stepv{
                std::dynamic_pointer_cast<intermediate::FloatDataType>(stepi)
                ? *stepi->dataAsFloat() : float(*stepi->dataAsInt())
            };
			int size = int(ceil((endv - startv) / stepv));
			float *data = new float[size];
			int i; float v;
			for (i = 0, v = startv; i < size; i++, v += stepv)
				data[i] = v;
            type = DTypeSP{new intermediate::ArrayDataType(blt()->FLOAT, size)};
            return ValueSP{new Value{type, (void*) data}};
        } else if (std::dynamic_pointer_cast<intermediate::IntDataType>(type)) {
            int startv{*starti->dataAsInt()}, endv{*endi->dataAsInt()},
                stepv{*stepi->dataAsInt()};
            int size = (endv - startv + stepv - 1) / stepv;
            int *data = new int[size];
            for (int i = 0, v = startv; i < size; i++, v += stepv) {
                data[i] = v;
            }
            type = DTypeSP{new intermediate::ArrayDataType(blt()->INT, size)};
            return ValueSP{new Value{type, (void*) data}};
        }
    }
}

ValueSP grammar::FuncCall::getValue(ScopeSP context) {
    ScopeSP call = context->lookupFunc(name);
    CommandSP fc{new Command(call)};

    if (ins->getExps().size() != call->getIns().size()) {
        // TODO: Argument mismatch error.
        return nullptr;
    }
    for (int i = 0; i < ins->getExps().size(); i++) {
        fc->addInput(ins->getExps()[i]->getValue(context));
    }

    if (outs->getOutputs().size() != call->getOuts().size()) {
        // TODO: Output mismatch error.
        return nullptr;
    }
    ValueSP toReturn;
    for (int i = 0; i < outs->getOutputs().size(); i++) {
        std::shared_ptr<Output> rval = outs->getOutputs()[i];
        switch(rval->getType()) {
        case RetOut::TYPE_CONST:
            toReturn = ValueSP{new Value(call->getOuts()[i]->getType())};
            context->declareTempVar(toReturn);
            fc->addOutput(toReturn);
            break;
        case NoneOut::TYPE_CONST:
            fc->addOutput(nullptr);
            break;
        case VarAccessOut::TYPE_CONST:
            fc->addOutput(rval->getExp()->getValue(context));
            break;
        }
    }
    context->addCommand(fc);
    return toReturn ? toReturn : nullptr;
}

}
}