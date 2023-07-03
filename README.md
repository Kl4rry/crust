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
| Export                          | ✅       | ✅    |
| Alias                           | ✅       | ✅    |
| Lists                           | ✅       | ✅    |
| Indexing                        | ✅       | ✅    |
| Slicing                         | ❌       | ❌    |
| Maps                            | ✅       | ✅    |
| Tables                          | ✅       | ✅    |
| Pipes                           | ✅       | ✅    |
| Glob                            | ✅       | ✅    |
| Questionmark / single char glob | ✅       | ✅    |
| Bracket globing char            | ✅       | ✅    |
| Subexpressions                  | ✅       | ✅    |
| Assign and modify += *= etc     | ✅       | ✅    |
| Run command as background job   | ❌       | ❌    |
| Escapes sequences               | ✅       | ✅    |
| Closures                        | ❌       | ❌    |

## TODO
* Add span info to ast.
* Add serde support to value.

### Standard functions
| Name       | Completed                   | About                         |
| ---------- | --------------------------- | ----------------------------- |
| clear      | ✅                           | Clear screen                  |
| cd         | ✅                           | Change working directory      |
| exit       | ✅                           | Exit shell                    |
| echo       | ✅                           | Echo value back               |
| pwd        | ✅                           | Print working directory       |
| import     | ✅ (Not locking / integrity) | Import code from file or URL  |
| open       | ✅                           | Open url with default program |
| read/input | ✅                           | Read user input from stdin    |
| load       | ✅                           | Read data from file           |
| save       | ✅                           | Save data to file             |


(More functions to come)

### Other features
| Feature              | Completed |
| -------------------- | --------- |
| Custom prompt        | ✅         |
| Starship integration | ✅         |

### Starship integration
```bash
import https://raw.githubusercontent.com/Kl4rry/crust/main/libs/starship.crust
```

## Todo
Columns and indexing when passing arguments.  
Allow column with number name.  
Add builtins and functions to help when calling unknown command.  
Rework defing functions to use cli parser.  
Add sub commands to cli parser.  
Add custom ls command.  
Convert statements into expressions.  
Allow expressions to be start of pipeline.  
Test scoped cd.  
Multi line Table entry.  
Make it possible to use input from pipeline in user defined function.  
Better error for unclosed delimiters.  

## Low prio todo
Make command pipelines lazy.  
Create a better line writer then rustyline.  
Fish-like selector for completer.  

## Bugs
A bunch of places convert values to string where it should throw an hard error instead.  
Comparison operator chaining is currently permitted but should not be.  

## Questions to be answered
Should lists expand to multiple arguments when passed to a function. Should this depend on if it is an internal or external command?  
Should return take a expr as optional parameter? What does even return mean as functions dont have traditional return values?  

