#pragma once

#include <boost/core/enable_if.hpp>
#include <boost/variant.hpp>

#include "util/aliases.hpp"

namespace waveguide {
namespace util {

template<typename ChildClass, typename DataContainer>
class static_visitor: boost::static_visitor<> {
private:
    template<typename T>
    struct has_basic_method {
    private:
        typedef std::true_type yes;
        typedef std::false_type no;
        
        template<typename U> static auto test(int) -> decltype(
            std::declval<ChildClass>()(std::declval<U>()),
            yes());
        template<typename> static no test(...);

    public:
        static constexpr bool value
            = std::is_same<decltype(test<T>(0)),yes>::value;
    };

protected:
    SP<DataContainer> data;

    virtual void on_start() const = 0;

    template<typename Visitable>
    typename boost::disable_if<has_basic_method<Visitable>, void>::type
    recurse(Visitable const&to_convert) const {
        ChildClass child{};
        child.data = data;
        boost::apply_visitor(child, to_convert);
    }

    template<typename Visitable>
    typename boost::enable_if<has_basic_method<Visitable>, void>::type
    recurse(Visitable const&to_convert) const {
        (*static_cast<const ChildClass*>(this))(to_convert);
    }

public:
    template<typename T>
    void start(T const&start_item) {
        on_start();
        recurse(start_item);
    }
};

}
}