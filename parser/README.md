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
* Continue
* Return ✅
* Assignment ✅
* Variable declaration ✅
* Const variables ❌
* Export ❌
* Alias ❌
* Lists ❌
* Indexing ❌
* Maps ❌
* Pipes ❌
* Redirects ❌
* Glob ✅
* Questionmark / single char glob ❌
* Bracket globing char ❌ (Still not sure about this one)
* $(expr) ❌

# Builtin thoughts
There needs to be a function that returns the type of an expression.  
The type function should return a special type id.  
  
Current plan for function calls are to treat them syntactically the same as calls to external programs.  
$(expr) syntax to be considered. 

Comparison operator chaining is currently permitted but should not be.