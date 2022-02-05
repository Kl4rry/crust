# Crust
Crust is a experimental crossplatform exotic shell.  
This readme contains notes to myself.

## Progress
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
| Subexpr $(expr)                 | ✅                            | ✅    |
| Assign and modify += *= etc     | ✅                            | ✅    |
| run in background using &       | ❌                            | ❌    |
| Escapes sequences               | ❌                            | ❌    |

## Todo
Capture output stream in each stack frame.

## Bugs
Comparison operator chaining is currently permitted but should not be.  
Exit status is never set/checked.
Null should be filter out when combining outputstreams (Maybe make a combine method on output stream and/or stream).  
Streams cannot be used like normal return values. They should be unpacked when used in a sub expr.  
Ctrl-C only works for external programs not infinte loops. Must insert points where the script can be preempted.  
Arguments are always passed as strings to functions they should be passed as values (Building my own CLI arg parser?).  
Check for overflowing, div by zero and use wrapping/checked arthimetic operations.  

## Questions to be answered
Should lists expand to multiple arguments when passed to a function. Should this depend on if it is an internal or external command?  
Should return take a expr as optional parameter?
