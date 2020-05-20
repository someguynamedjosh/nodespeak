Make sure everything has appropriate errors.
Add type checking for unary and binary operators.
Think of a better panic message to replace "bad AST", maybe "bad ast"?
Check that all the error messages make sense.
When adding a source with an existing name, replace that source instead of adding a new identically named source
Make it compile correctly with the no-resolved feature
Nice error for multiple identically named function parameters.
Fix bug where resolve phase fails when a variable is defined inside a loop.
Allow syntax like: thing[5]:DIMS[3]