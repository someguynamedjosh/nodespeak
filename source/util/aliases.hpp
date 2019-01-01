#include <memory>

namespace waveguide {
    namespace intermediate { }
    namespace intr = intermediate;

    template<typename T>
    using SP = std::shared_ptr<T>;
}