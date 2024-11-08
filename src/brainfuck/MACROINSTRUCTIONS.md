# Macroinstructions
This is meant to explain the various types of macro-instructions I introduce to the intermediate representation (IR).

## Requirements
A valid macroinstruction definition must meet the following requirements:
- Surrounded by parentheses
- Contains no reserved characters (the 8 characters with existing meanings)
- Uses a unique separator character of some kind

## Alternate Opcodes
Since some macro-instructions need to refer to a regular instruction (i.e. the repetition one) we define the following
alternative opcodes for this specific use case:
- `+` => `p` (plus)
- `-` => `m` (minus)
- `>` => `f` (forward)
- `<` => `b` (backward)
- `.` => `o` (output)
- `,` => `i` (input)
- `[` => `{`
- `]` => `}`
`[` and `]` probably won't need alternate codes but these fit so well I thought I'd define them anyway

## Current Macroinstructions
- `REPEAT`: represented in the IR by `(<count>*<alt_opcode>)`. Examples:
    - `(12*f)` means the same as `>>>>>>>>>>>>`
    - `(8*m)` means the same as `--------`
- `BOUNDCHECK`: represented in the IR by `(<max_left>;<max_right>)`, tells the interpreter to check whether going
`max_left` cells to the left of the starting location, or `max_right` cells to the right of the starting location, will
cause an out of bounds error. Examples:
    - `(5;2)` would throw an error if `dp <= 4` since `4 - 5 < 0`, or if `dp >= 29998` since `29998 + 2 > 29999`
    - `(100;0)` would throw an error if `dp <= 99` since `99 - 100 < 0`

