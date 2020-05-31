Make sure everything has appropriate errors.
Add type checking for unary and binary operators.
Check that all the error messages make sense.
Allow syntax like: thing[5]:DIMS[3]
Some way to combine arrays of boolean values into a single bool.

# Critical bugs, maybe check here when the compiler crashes instead of spending
# 30 minutes whittling down a minimal case
Fix bug where resolve phase fails when a variable is defined inside a loop.
Make it compile correctly with the no-resolved feature
Nice error for multiple identically named function parameters.