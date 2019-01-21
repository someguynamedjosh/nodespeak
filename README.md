# Waveguide

Waveguide is a language designed to write short, number-crunching applications
that can be JIT-compiled to get the most performance out of them as possible.
For example, one use case would be for filters in image editing applications.
Most filters have a handful of parameters that the user can tweak, and the
filter is then applied across a large set of data. By using JIT instead of AOT
compilation, the filter can be optimized depending on the parameters the user
inputs.

This is currently *very* work in progress, and there is no fully working version
yet.