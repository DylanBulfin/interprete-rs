# Optimizations
Notes about the various optimizations I used

## Macroinstructions
This is meant to explain the various types of macro-instructions I introduce to the intermediate representation (IR).

### Requirements
A valid macroinstruction definition must meet the following requirements:
- Surrounded by parentheses
- Contains no reserved characters (the 8 characters with existing meanings)
- Uses a unique separator character of some kind

### Alternate Opcodes
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

### Current Macroinstructions
- `REPEAT`: represented in the IR by `(<count>*<alt_opcode>)`. Examples:
    - `(12*f)` means the same as `>>>>>>>>>>>>`
    - `(8*m)` means the same as `--------`
- `BOUNDCHECK`: represented in the IR by `(<max_left>;<max_right>)`, tells the interpreter to check whether going
`max_left` cells to the left of the starting location, or `max_right` cells to the right of the starting location, will
cause an out of bounds error. Examples:
    - `(5;2)` would throw an error if `dp <= 4` since `4 - 5 < 0`, or if `dp >= 29998` since `29998 + 2 > 29999`
    - `(100;0)` would throw an error if `dp <= 99` since `99 - 100 < 0`
- `LOOPCHECK`: represented in the IR by `(h)` or `(oh)` (h for 'halting' or 'hanging', while o is an alt opcode for the
output instruction). Tells the interpreter to check if current memory value is 0, and either move on or hang forever 
(potentially while printing the current memory location infinitely)
    - `(h)` hangs forever if the value of the current memory location is non-zero, otherwise does nothing
    - `(oh)` prints the value of the current memory location infinitely if it is non-zero, otherwise does nothing


## Loop optimizations
Loop instructions don't have any meaning on their own like every other instruction, which makes optimizing them much
more difficult. Still, there are a couple patterns we can identify:
- Loops with no memory-changing commands inside
    - `[]` is either an infinite loop or a no-op, depending on the value of the current memory cell when it's seen
    - `[.*]` e.g. a loop with any number of `.` inside, is either an infinite loop that prints the current memory value
    forever, or a no-op
- Separate loops with no memory-changing commands between them
    - `][`: The first loop exits only if the current mem value is `0`, but if this is the case the second loop will be
    skipped. This means the second loop can be totally removed
    - `].*[`: Same as above except the print commands are maintained
These optimizations should be made after the math and data pointer optimizations but before compressing sequences. This
is so we can correctly identify sequences of `.` without parsing the repeat macroinstruction
