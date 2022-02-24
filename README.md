# Crust
Crust is a experimental crossplatform exotic shell.  
This readme contains notes to myself.

## Progress
### Language features
| Syntax                          | Parsing | Eval |
| ------------------------------- | ------- | ---- |
| Literals                        | ✅       | ✅    |
| Binary operators                | ✅       | ✅    |
| Unary operators                 | ✅       | ✅    |
| Function/Command calls          | ✅       | ✅    |
| Blocks                          | ✅       | ✅    |
| Function Declarations           | ✅       | ✅    |
| If                              | ✅       | ✅    |
| loop                            | ✅       | ✅    |
| while                           | ✅       | ✅    |
| for                             | ✅       | ✅    |
| Break                           | ✅       | ✅    |
| Continue                        | ✅       | ✅    |
| Return                          | ✅       | ✅    |
| Assignment                      | ✅       | ✅    |
| Variable declaration            | ✅       | ✅    |
| Export                          | ✅       | ❌    |
| Alias                           | ✅       | ✅    |
| Lists                           | ✅       | ✅    |
| Indexing/Slicing                | ❌       | ❌    |
| Maps/Tables                     | ❌       | ❌    |
| Pipes                           | ✅       | ✅    |
| Glob                            | ✅       | ✅    |
| Questionmark / single char glob | ✅       | ✅    |
| Bracket globing char            | ✅       | ✅    |
| Subexpr $(expr)                 | ✅       | ✅    |
| Assign and modify += *= etc     | ✅       | ✅    |
| run in background using &       | ❌       | ❌    |
| Escapes sequences               | ❌       | ❌    |

### Standard functions
| Name   | Completed       |
| ------ | --------------- |
| clear  | ✅               |
| cd     | ✅               |
| exit   | ✅               |
| echo   | ✅               |
| pwd    | ✅               |
| import | ✅ (Not locking) |
| open   | ❌               |

(More functions to come)

### Other features
| Feature              | Completed |
| -------------------- | --------- |
| Custom prompt        | ✅         |
| Starship integration | ✅         |

## Todo
Propagate errors better with streams.  

## Bugs
A bunch of places convert values to string where it should throw an hard error instead.  
Comparison operator chaining is currently permitted but should not be.  
Streams cannot be used like normal return values. They should be unpacked when used in a sub expr.  
Check for overflowing, div by zero and use wrapping/checked arthimetic operations.  

## Questions to be answered
Should lists expand to multiple arguments when passed to a function. Should this depend on if it is an internal or external command?  
Should return take a expr as optional parameter? What does even return mean as functions dont have traditional return values?  

## Ideas
All env variables should be stored in the same way as normal variable only with a flag. When a process is started all env variables in scope should be collected and passed to the child.

