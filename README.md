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
| Closures                        | ✅       | ✅    |

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
Add builtins and functions to help when calling unknown command.  
Rework defing functions to use cli parser.  
Add sub commands to cli parser.  
Convert statements into expressions.  
Better error for unclosed delimiters.  

## Low prio todo
Make command pipelines lazy.  
Create a better line writer than rustyline.  
Fish-like selector for completer.  

## Bugs
A bunch of places convert values to string where it should throw an hard error instead.  

## Questions to be answered
Should return take a expr as optional parameter? What does even return mean as functions dont have traditional return values?  

