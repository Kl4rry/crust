# Features
* Literals ✅
* Binary operators ✅
* Unary operators ✅
* Function/Command calls ✅
* Blocks ✅
* Function Declarations ✅
* Control flow ✅
* Loops ✅
* Break ✅
* Continue ✅
* Return ✅
* Assignment ✅
* Variable declaration ✅
* Export ✅
* Alias ✅
* Lists ❌
* Indexing ❌
* Maps ❌
* Pipes ✅ (Not |& aka stderr pipe)
* Redirects ✅
* Glob ✅
* Questionmark / single char glob ✅
* Bracket globing char ✅
* $(expr) ✅
* Assign and modify += *= etc ❌
* run in background using & ❌
* Escapes sequences ❌

# Builtin thoughts
There needs to be a function that returns the type of an expression.  
The type function should return a special type id.  
  
Current plan for function calls are to treat them syntactically the same as calls to external programs.  

Comparison operator chaining is currently permitted but should not be.