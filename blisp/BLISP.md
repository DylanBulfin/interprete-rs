# BLisp
Short for 'Basic Lisp', pronounced 'bee-lisp' so as to rhyme with 'ELisp', this is a custom Lisp dialect with a very
limited instruction set. I am not very familiar with Lisp so it may or may not end up looking anything like other Lisp
variants. The only thing that I'm certain will be like other dialects is the control flow/calculation order

## Control Flow
Each BLisp program is a single lisp statement, which simplifies the control flow. If you want multiple statements you
must construct a list from them (explained later on)

## Types
I want this language to have a strict type system, analogous to Haskell or Rust. The types I will support:
- `int` (internally stored as `i64`) is a signed int type
- `uint` (internally stored as `u64`) is an unsigned int type
- `float` (internally stored as `f64`) is a signed floating-point type
- `bool` (internally stored as `bool`) is a boolen type
- `char` (internally stored as `u8`) is an ASCII character
- `list<ty>` (internal representation tbd) is a list of type `ty` (which can be any valid type, including another list)
- `string` (internal representation tbd) is an alias for `list<char>`
- `tuple<ty1, ty2>` is a 2-tuple, where the first value is of type `ty1` and the second is of type `ty2`
- `unit`/`()` represents an empty value. Mostly used to make sequences of functions easier

### Literals

#### Integers
Any numeric literal with no period is treated as an integer literal by default. They default to `int` type unless 
`uint` is required in context. To force a literal to be `uint` append a `u` to it:
- `(1)` is `int`
- `(1u)` is `uint`
- `(-1)` is `int`
- `(-1u)` is a compilation error

#### Floats
Any numeric literal with a period is treated as a float literal. To force a numeric literal to be a float, append `f` to
it:
- `(1.)` is `float`
- `(1.0)`is `float`
- `(1f)` is `float`
- `(.1)` is a compilation error since floats must have at least one digit preceding the decimal point
- `(1.0u)` is a compilation error since floats can not be coerced to ints (unsigned or otherwise)

#### Boolean
As usual, `true` and `false` are used

#### Strings
String literals are always sequences of ASCII data enclosed by double quotes `""`. Examples:
- `("ABC")`
- `("AOTIENSR_RS_D_SD_*SRT)`
- `('ABC')` is a compilation error since double quotes were not used

#### Chars
By default, any ASCII char surrounded by single quotes '' is a char literal. You can also force a numeric literal to be
a char by appending `c`:
- `('a')` is `char`
- `("a")` is `string`
- `(57c)` is `char` and is equivalent to `('9')`
- `(256c)` is a compilation error since the max value of an ASCII char is 255 (including extended ASCII)

#### Lists
Lists are enclosed by `[]` and their type is determined by the types of the members. If the members are not all of the
same type and they can't be coerced to the same type, the list declaration is invalid:
- `[1 2 3]` is `list<int>`
- `[1u, 2, 3]` is `list<uint>` since the second and third elements are coerced to `uint` to match the first
- `[1f, 2, 3]` is `list<float>` since the second and third elements are coerced to `float` (may change)
- `[true, false]` is `list<bool>`
- `[[1u, 2, 3], [4, 5, 6]]` is `list<list<uint>>`, the second list is coerced to `list<uint>`
- `[]` is a compilation error since the list's type can't be determined without at least one value (may change later)
- `['a', "bcd"]` is a compilation error since `char` and `string` are incompatible types
- `[1, 1c]` is a compilation error because there is no implicit coercion from int literal to char (may change)

#### (2-)Tuples
Tuples are enclosed by `()` always, and consist of 2 comma-separated values, potentially of different types
- `(1, 'a')` is `tuple<int, char>`
- `(true, [1, 2, 3u])` is `tuple<bool, list<uint>>`

### Type Coercion 
To allow for coercion we must define a couple abstract types and a hierarchy. To start we should look at which type 
coercions are possible, and in what contexts:
- Any numeric literal with a decimal point is a `float` and non-coercible
- Any numeric literal with a negative sign and no decimal point is `negnum` type, which defaults to `int` and can be 
coerced to `float` or `int`
- Any numeric with neither a negative sign nor a decimal point is a `num` type which defaults to `int` and can be
coerced to `int`, `uint`, or `float`
- While there is a suffix that turns integer literals into chars, at least for now there is no implicit coercion 
between the two. E.g. `[1c, 'd']` is fine but `[1c, 2]` or `['d', 2]` are errors

## Variables
Variables are values bound to a name, have an associated type, and can be used in any context that a literal of the
same type would be valid (and a few others). 

## Functions

### Math
- `+` or `add`
    - `(+ 1 2) = (add 1 2) = 3` 
    - `(+ T T) -> T` is defined for `T: int | uint | float`
- `-` or `sub`
    - `(- 2 1) = (sub 2 1) = 1`
    - `(- T T) -> T` is defined for `T: int | uint | float`
- `*` or `mul`
    - `(* 2 3) = (mul 2 3) = 6`
    - `(* T T) -> T` is defined for `T: int | uint | float`
- `/` or `div`
    - `(/ 10 2) = (div 11 2) = 5` since this would be integer division
    - `(/ 9.0 2.0) = (div 9.0 2.0) = 5.0`
    - `(* T T) -> T` is defined for `T: int | uint | float`

### I/O
- `.` or `write`
    - `(. "ABC")` and `(print "ABC")` both print "ABC" to stdout and return "ABC"
    - `(. T) -> T` is defined for `T: string`
- `,` or `read`
    - `(,)` and `(read)` both read from stdin and return the value as `string`
    - `(,) -> T` is defined for `T: string`

### Control Flow
- `?` or `if`
    - `(if true "option1" "option2") = (? false "option2" "option1") = "option1`
    - `(? T U U) -> U` is defined for `T: bool, U: any`
- `while`
    - `(while true (print "ABC"))` prints "ABC" forever
    - `(while T U) -> ()` is defined for `T: bool, U: any`

### Boolean Operations
- `==` or `eq`
    - `(== 1 1) = (eq 'a' 'a') = true`
    - `(== T T) -> bool` defined for all default types
- `<>` or `neq`
- `<=` or `leq`
- `>=` or `geq`
- `<` or `lt`
- `>` or `gt`
- `&&` or `and`
- `||` or `or`

### Collection methods
- `++` or `concat` 
    - `(++ [1, 2, 3] [4, 5, 6]) = (concat [1] [2, 3, 4, 5, 6]) = [1, 2, 3, 4, 5, 6]`
    - `(++ list<T> list<T>) -> list<T>` defined for `T: any` (which includes strings)
- `:` or `prepend`
    - `(: 1 [2, 3, 4]) = (prepend 1 [2, 3, 4]) = [1, 2, 3, 4]`
    - `(: T list<T>) -> list<T>` defined for `T: any` (which includes strings, where `T: char`)
- `take`
    - `(take 2 [1, 2, 3, 4, 5]) = [1, 2]`
    - `(take T list<U>) -> list<U>` is defined for `T: uint, U: any`
- `split`
    - `(split 3 [1, 2, 3, 4, 5]) = ([1, 2, 3], [4, 5])`
    - `(split T list<U>) -> (list<U>, list<U>)` is defined for `T: uint, U: any`
