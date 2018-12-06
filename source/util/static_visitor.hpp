#pragma once

#include <boost/core/enable_if.hpp>
#include <boost/variant.hpp>

namespace waveguide {
namespace util {

template<typename V>
struct static_visitor: boost::static_visitor<> {
    template<typename T>
    struct has_visit_method {
    private:
        typedef std::true_type yes;
        typedef std::false_type no;
        
        template<typename U> static auto test(int) -> decltype(
            std::declval<U>().apply_visitor(std::declval<V>()), 
            yes());
        template<typename> static no test(...);

    public:
        static constexpr bool value
            = std::is_same<decltype(test<T>(0)),yes>::value;
    };

    virtual const V* get_recurse_object() const = 0;

    template<typename Visitable>
    typename boost::enable_if<has_visit_method<Visitable>, void>::type
    recurse(Visitable const&to_convert) const {
        boost::apply_visitor(*get_recurse_object(), to_convert);
    }

    template<typename Visitable>
    typename boost::disable_if<has_visit_method<Visitable>, void>::type
    recurse(Visitable const&to_convert) const {
        (*get_recurse_object())(to_convert);
    }
};

}
}