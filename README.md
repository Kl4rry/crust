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
Comparison operator chaining is currently permitted but should not be.

Unary operators have no precedence.