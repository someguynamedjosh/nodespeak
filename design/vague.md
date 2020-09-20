# (Internal) Vague Program Description

## Scope
An object storing names of variables and macros that are valid
in the scope. Since both of them will eventually be able to be used
interchangeably in expressions, they should both be grouped together as an enum.
Think it should be called `Entity` because that sounds generic enough. Scopes
can also have parent scopes so that if they do not have a definition for a
particular symbol, they can look it up in the parent.

## Code Block
Contain a series of statements. Store a reference to the scope they draw from.
The reason to separate scopes and code blocks is because sometimes there are
bits of code the user writes that require several statements to describe but are
ultimately not full-on scopes (such as using expressions to declare the type of
a variable.)

## Statements
Have a list of input and output entities. Point to a macro entity that is
used to perform the computation. 

## Entities
Entity is an enum encapsulating macros and variables. Variables have data
types, macros have a scope and a code block and lists of variables defined
in that scope that are inputs and outputs.

## Data Types
An enum with elementary data types and an array data type.