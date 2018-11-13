#ifndef _WAVEGUIDE_CONVERT_UTILS_H_
#define _WAVEGUIDE_CONVERT_UTILS_H_

#include <memory>

#include "intermediate/Builtins.h"
#include "intermediate/DataType.h"
#include "intermediate/Scope.h"
#include "intermediate/Value.h"

namespace waveguide {
namespace convert{

typedef intermediate::DataType DataType;
typedef intermediate::Scope Scope;
typedef intermediate::Value Value;
typedef intermediate::Command Command;
typedef intermediate::Augmentation Augmentation;

typedef std::shared_ptr<DataType> DTypeSP;
typedef std::shared_ptr<Scope> ScopeSP;
typedef std::shared_ptr<Value> ValueSP;
typedef std::shared_ptr<Command> CommandSP;
typedef std::shared_ptr<Augmentation> AugmentationSP;

DTypeSP pickBiggerType(DTypeSP a, DTypeSP b);

std::shared_ptr<intermediate::Builtins> blt();

}
}

#endif /* _WAVEGUIDE_CONVERT_UTILS_H_ */