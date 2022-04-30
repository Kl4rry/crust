# Intro
Crust is a imperative shell scripting language with a strong a dynamic type system. Crust is designed to be a glue language like bash, perl or python while also providing a user friendly shell REPL experience. 

## Type system
Crust is dynamically typed meaning a variable can be any data type but it is also strongly typed which means variables are rarely coerced into a different type. All types are immutable and Crust therefore mixes in a lot of functional patterns.

# Data types
## Integers
Integers are whole number 1, 5, 10 etc. They are currently represented as a 64-bit signed integer that wraps when it is overflowed.

## Floats
A float can represent decimal numbers like 1.3, 3.0 and 100.2. The are currently represented as a 64 bit float internally.

## Booleans
Booleans represent `true` or `false`.

## Strings
Strings are sequences of characters. All strings in Crust are guaranteed to be valid utf-8. Strings can be created a number of ways. The simplest is with single quotes.

```bash
'a string'
```

Strings can also be created using double quotes.
```bash
"a string"
```
Double quoted strings have support for escape sequences like `\n`.
```bash
# The \n is converted to a newline.
"a line\n"
```
The available escape sequences are:
| Sequences | Character       |
| --------- | --------------- |
| `\n`      | Newline         |
| `\t`      | Tab             |
| `\0`      | Null            |
| `\r`      | Carriage return |
| `\s`      | Space           |

### String interpolation
Another feature of double quoted strings is string interpolation.
```bash
$x = "world"
"Hello $x!"
# Outputs: Hello world!

"result: (1 + 3 * 2)"
# Outputs: result: 7
```

## Ranges
Ranges are sequences of integers. The are created like this: `2..15`

## Lists
Lists hold a sequence of values.
```bash
[1, 2, 3, "abc"]
# Output:
╭───┬─────╮
│ 0 │   1 │
│ 1 │   2 │
│ 2 │   3 │
│ 3 │ abc │
╰───┴─────╯
```
Lists can be indexed into like this:
```bash
$x = [1, 2, 3, "abc"]

$x[1]
# Output: 2

$x[-1]
# Output: abc
```

## Maps
A map is an associative array that holds a sequence of key value pairs where the key is always a string.
```bash
@{"abc": 123, "foo": "bar", 5: 10}
╭─────┬─────╮
│ abc │ 123 │
│ foo │ bar │
│   5 │  10 │
╰─────┴─────╯
```
Values can be retrieved from a map like this:
```bash
$x = @{"abc": 123, "foo": "bar", 5: 10}

$x.abc
# Output: 123
```

## Tables
Tables is a more convenient way to represent a list of maps. They can be both indexed into like a list and you can get a whole column from it just like map. When a list literal contains only maps it is automatically converted to a table.

```bash
$x = [
    @{"abc": 123, "foo": "bar", "oof": 10},
    @{"abc": "hello", "foo": "value", "oof": 5379}
]

echo $x
# Output:
╭───┬───────┬───────┬──────╮
│ # │  abc  │  foo  │ oof  │
├───┼───────┼───────┼──────┤
│ 0 │   123 │   bar │   10 │
│ 1 │ hello │ value │ 5379 │
╰───┴───────┴───────┴──────╯

$x.abc
# Output:
╭───┬───────╮
│ 0 │   123 │
│ 1 │ hello │
╰───┴───────╯

$x[0]
# Output:
╭─────┬─────╮
│ abc │ 123 │
│ foo │ bar │
│ oof │  10 │
╰─────┴─────╯
```

# Control flow
Crust has control flow primitives that should feel familiar to anyone who has used an imperative scripting language before.
## Conditional
```bash
$x = 10
if $x < 5 {
    echo "less then"
} else if $x > 5 {
    echo "more then"
} else {
    echo "equal"
}
```
## Loops
For loops loop over a sequence.
```bash
for $i in 0..10 {
    echo "iteration: $i"
}
```

While loops loop while a conditional expression is true.
```bash
$x = 0
while $x < 10 {
    echo $x
    $x += 1
}
```

Bare loops loop until they are broken out of.
```bash
$y = 0
loop {
    echo $y
    $y += 1
    if $y > 20 {
        break
    }
}
```

Continue can be used to skip to the next iteration of the loop.
```bash
for $i in 0..10 {
    if $i % 2 == 0 {
        continue
    }
}
```

## Scopes
Variables in Crust are bound to the closest scope.
```bash
{
    $x = 10;
    echo $x
}
echo $x
# This does not work because $x is no longer in scope
```
The `let` keyword is used to create a new variable binding in the current scope.
```bash
$x = 20
{
    let $x = 10;
    echo $x
    # Output: 10
}
echo $x
# Output: 20
```
## Environment variables
Crust has support for settings environment variables which are passed down to child processes. Environment variables are scoped just like regular variables
In Crust environment variables are created using the export keyword.
```bash
export $x = "foo"

printenv x
# Output: foo

$x = "bar"
printenv x
# Output: bar

# Scoped env var
{
    export $y = "hello";
    printenv y
    # Output: hello
}
printenv y
# The output is empty because y does not exist anymore. 
```
printenv is a external program which prints the value of environment variables.
