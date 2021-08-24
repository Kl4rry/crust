# Features
* Literals ✅
* Binary operators ✅
* Unary operators ✅
* Commands ✅
* Function calls ❌
* Blocks ✅
* Function Declarations ❌
* Control flow ❌
* Break ✅
* Continue
* Return ❌
* Assignment ✅
* Variable declaration ✅
* Const variables ❌
* Export ❌
* Alias ❌
* Lists ❌
* Maps ❌
* Pipes ❌
* Redirects ❌
* Glob ✅
* Questionmark / single char glob ❌
* Bracket globing char ❌

# Builtin thoughts
There needs to be a function that returns the type of an expression.  
The type function should return a special type id.  
  
Current plan for function calls are to treat them syntactically the same as calls to external programs.  
$(command) syntax to be considered.