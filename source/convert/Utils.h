#ifndef _WAVEGUIDE_CONVERT_UTILS_H_
#define _WAVEGUIDE_CONVERT_UTILS_H_

#include <memory>

#include "intermediate/DataType.h"
#include "intermediate/Scope.h"
#include "intermediate/Value.h"

namespace waveguide {
namespace convert{

typedef std::shared_ptr<intermediate::DataType> DTypeSP;
typedef std::shared_ptr<intermediate::Scope> ScopeSP;
typedef std::shared_ptr<intermediate::Value> ValueSP;

}
}

#endif /* _WAVEGUIDE_CONVERT_UTILS_H_ */