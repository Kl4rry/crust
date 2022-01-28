# Crust
This readme contains notes to myself.

## Parsing
| Syntax                          | Parsing                      | Eval |
| ------------------------------- | ---------------------------- | ---- |
| Literals                        | ✅                            | ✅    |
| Binary operators                | ✅                            | ✅    |
| Unary operators                 | ✅                            | ✅    |
| Function/Command calls          | ✅                            | ❌    |
| Blocks                          | ✅                            | ✅    |
| Function Declarations           | ✅                            | ✅    |
| If                              | ✅                            | ✅    |
| loop                            | ✅                            | ✅    |
| while                           | ✅                            | ✅    |
| for                             | ✅                            | ✅    |
| Break                           | ✅                            | ✅    |
| Continue                        | ✅                            | ✅    |
| Return                          | ✅                            | ✅    |
| Assignment                      | ✅                            | ✅    |
| Variable declaration            | ✅                            | ✅    |
| Export                          | ✅                            | ❌    |
| Alias                           | ✅                            | ✅    |
| Lists                           | ✅                            | ✅    |
| Indexing/Slicing                | ❌                            | ❌    |
| Maps                            | ❌                            | ❌    |
| Pipes                           | ✅ (Not \| & aka stderr pipe) | ❌    |
| Glob                            | ✅                            | ✅    |
| Questionmark / single char glob | ✅                            | ✅    |
| Bracket globing char            | ✅                            | ✅    |
| $(expr)                         | ✅                            | ✅    |
| Assign and modify += *= etc     | ✅                            | ✅    |
| run in background using &       | ❌                            | ❌    |
| Escapes sequences               | ❌                            | ❌    |

## Todo
Capture output stream in each stack frame.

## Bugs
Comparison operator chaining is currently permitted but should not be.  
Parser can overflow when parsing number bigger then u128::MAX.
Exit status is ignored everywere.
Output is only printed when the program reaches its end because all output is caputured even if it is not in a subexpr or pipe.
Null should be filter out when combining outputstreams (Maybe make a combine method on output stream and/or stream).

## Builtin thoughts
Lists should become multiple arguments

## Design rules
Data structures cannot be cyclic. This means that lists are cannot hold lists.
