# Crust
This readme contains notes to myself.

## Parsing
| Syntax                          | Parsing                      | Eval |
| ------------------------------- | ---------------------------- | ---- |
| Literals                        | ✅                            | ✅    |
| Binary operators                | ✅                            | ✅    |
| Unary operators                 | ✅ (Precedence broken)        | ✅    |
| Function/Command calls          | ✅                            | ❌    |
| Blocks                          | ✅                            | ✅    |
| Function Declarations           | ✅                            | ✅    |
| If                              | ✅                            | ✅    |
| loop                            | ✅                            | ✅    |
| while                           | ✅                            | ✅    |
| for                             | ✅                            | ❌    |
| Break                           | ✅                            | ✅    |
| Continue                        | ✅                            | ✅    |
| Return                          | ✅                            | ✅    |
| Assignment                      | ✅                            | ✅    |
| Variable declaration            | ✅                            | ❌    |
| Export                          | ✅                            | ❌    |
| Alias                           | ✅                            | ✅    |
| Lists                           | ✅                            | ✅    |
| Indexing/Slicing                | ❌                            | ❌    |
| Maps                            | ❌                            | ❌    |
| Pipes                           | ✅ (Not \| & aka stderr pipe) | ❌    |
| Redirects                       | ✅                            | ❌    |
| Glob                            | ✅                            | ✅    |
| Questionmark / single char glob | ✅                            | ✅    |
| Bracket globing char            | ✅                            | ✅    |
| $(expr)                         | ✅                            | ✅    |
| Assign and modify += *= etc     | ❌                            | ❌    |
| run in background using &       | ❌                            | ❌    |
| Escapes sequences               | ❌                            | ❌    |

## Bugs
Comparison operator chaining is currently permitted but should not be.

Unary operators have no precedence.

## Builtin thoughts
Lists should become multiple arguments

## Design rules
Data structures cannot be cyclic. This means that lists are cannot hold lists.
