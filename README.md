# Interpreters
I have recently been thinking a lot about programming language design and how it affects their usage. I have also always
been interested in that side of computing but got turned off by my undergrad Compilers and Interpreters course. I want 
to ease back into it by making some interpreters for basic programming languages. Since most of these are either meant
for education or as jokes I'll provide context about them below.

## Brainfuck
Perhaps the most famous esoteric programming language, in part due to its name, Brainfuck is also one of the simplest to
understand (and one of the hardest to use). 

Brainfuck's memory/register layout is very simple:
- The input program, represented by an array of bytes
- An instruction pointer (IP) which holds the location of the next instruction to execute
- 30,000 1-byte cells, the memory of the program
- A data pointer (DP) which holds the location of the memory cell currently being acted on

There's a suprising amount of disagreement on how the memory cells should behave. Disagreements usually fit into the
following categories
1. The size, in bytes, of the memory cells
    - I refer to them as bytes and will implement them as such, but there are implementations with 32 bit cells, 64 bit
    cells, even cells with unbounded size (e.g. an arbitrary-precision integer type)
2. The behavior of mathematical operations around the limits of the byte, e.g. what is `0 - 1` or `255 + 1`
    - Wrapping is the easiest to implement and the simplest to understand so I will be using it. But 'saturating' 
    operations or runtime errors on overflow have been done before as well

Brainfuck has exactly 8 reserved characters, each of which corresponds to a single basic function. All other characters
in the input are ignored. The only syntactic restriction is that every `[` must have a matching `]` and vice versa.
(E.g. `]]`, `][`, and `[]]` are all invalid but `[]` and `[[][]]` are valid)
- `<` and `>` increment and decrement the data pointer respectively
- `+` and `-` increment and decrement the current memory cell (the cell pointed to by the DP register) respectively
- `.` outputs the byte in the current memory cell (usually to stdout/similar)
- `,` accepts one byte of input (usually from stdin) and store it in the current memory cell
- `[` if the byte in the current memory cell is 0, jump to right after the matching `]`, otherwise this is a no-op
- `]` if the byte in the current memory cell is non-zero, jump to right after the matching `[`

From this we see that all instructions except the branching ones are self-contained and can just be executed as we
come across them. The only analysis we need to do is to match `[]` pairs before we start the program.

### Additional Performance Considerations
As part of learning about writing my own interpreters I think some effort to make simple optimizations would be
educational
- `<>` is a no-op, as is `><`
    - When `<>` happens at `DP = 0` or `><` at `DP = 29,999` it's an error instead
    - For this we can statically replace these combos with a special new symbol that throws an error if at those
    locations
- `+-` and `-+` are no-ops always assuming wrapping behavior
    - For this we just statically remove the combos
- `[]` is either a no-op if the current cell is 0 or an inifinite loop otherwise
    - Same as with `<>`, statically replace this with a special instruction that hangs forever if current cell is not 0
- `[` at beginning of program is unconditional jump to matching `]`
    - Just delete everything up to and including the matching `]`
- `[-]` and `[+]` set the current memory cell to 0 and jump out of the loop
    - Replace these with special instruction that zeroes a register
- `][` means a skip of the second loop
    - E.g. the first loop doesn't exit until the current cell is 0, but this will cause the following loop to skip
For the first two I would like to generalize the logic; e.g. `++--` is a no-op as well but we would have to run my
strategy twice to reduce it fully. Perhaps I'll identify contiguous sequences of `+` and `-` and reduce them as normal.

I think the ultimate strategy is as follows:
- Get rid of as much control flow and basic inefficiencies as possible with the above optimizations
- Identify the innermost loops (any loops which contain no other loops)
- For each of them, perform more in-depth analysis, for example:
    - Identify contiguous sequences of `+` and `-`, reduce them as you need
        - This could be done without identifying inner loops, so is maybe 

I probably want to skip more complex static analysis if the input program doesn't have loops. In a program with no loops
the static analysis would likely take longer than the program would take to run

A loop with no `-`, `+`, `,`, `<`, or `>` is never-ending, there may be other patterns I can identify

I will first make a basic interpreter with no optimizations since simplicity of implementation is the entire reason
behind the language, but after this I will try adding the above optimizations to see how much of a performance increase
I see

### Transpilation/Compilation
The first attempts will be interpreters but eventually trans/compilation would be interesting. I would transpile into
C or Rust, and for compilation I would use LLVM
