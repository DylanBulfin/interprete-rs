use std::{
    collections::HashMap,
    io::{stdin, stdout, Read, Stdin, Stdout, Write},
};

pub const VALID_CHARS: [char; 8] = ['<', '>', '+', '-', '.', ',', '[', ']'];

/// Struct representing a Brainfuck program, storing the code, memory, pointers, bracket pairs, and
/// a reader and writer. `BrainfuckProgram::new(input:String)` is the standard way to create a new
/// program, which you can run with `prog.interpret_naive(). The function returns the memory block
/// when and if the program's execution is done
pub struct BrainfuckProgram<R, W>
where
    R: Read,
    W: Write,
{
    code: Vec<char>,
    mem: [u8; 30000],
    ip: usize,
    dp: usize,
    loops: HashMap<usize, usize>, // Matching pairs of brackets
    writer: W,
    reader: R,
}

impl<R, W> BrainfuckProgram<R, W>
where
    W: Write,
    R: Read,
{
    /// Create a new BrainfuckProgram, specifying both the reader and the writer.
    pub fn new_full(input: String, writer: W, reader: R) -> Self {
        let mut code = Vec::new();
        let mut stack = Vec::new();
        let mut loops = HashMap::new();

        for (i, c) in input
            .chars()
            .filter(|c| c.is_ascii() && VALID_CHARS.contains(c))
            .enumerate()
        {
            code.push(c);

            if c == '[' {
                stack.push(i);
            }
            if c == ']' {
                let matching = stack
                    .pop()
                    .expect("Mismatch in brackets, check that each [ has a matching ]");
                loops.insert(matching, i);
                loops.insert(i, matching);
            }
        }

        if !stack.is_empty() {
            panic!("Mismatch in brackets, check that each [ has a matching ]")
        }

        Self {
            code,
            loops,
            writer,
            reader,
            mem: [0; 30000],
            ip: 0,
            dp: 0,
        }
    }

    pub fn interpret_naive(mut self) -> [u8; 30000] {
        loop {
            if self.ip >= self.code.len() {
                // Reached end of selfram
                break;
            }
            match self.code[self.ip] {
                '<' => {
                    self.dp = self
                        .dp
                        .checked_sub(1)
                        .expect("Data pointer is 0, cannot decrement")
                }
                '>' => {
                    if self.dp < 29999 {
                        self.dp += 1
                    } else {
                        panic!("Data pointer is 29999, cannot increment")
                    }
                }
                '+' => self.mem[self.dp] = self.mem[self.dp].wrapping_add(1),
                '-' => self.mem[self.dp] = self.mem[self.dp].wrapping_sub(1),
                '.' => {
                    let cnt = self
                        .writer
                        .write(&self.mem[self.dp..self.dp + 1])
                        .expect("Unable to output data to configured writer");

                    if cnt != 1 {
                        panic!(
                            "Read {} bytes from configured reader, expected exactly 1",
                            cnt
                        );
                    }
                }
                ',' => {
                    let mut buf = [0u8];
                    let cnt = self
                        .reader
                        .read(&mut buf)
                        .expect("Unable to read from configured reader");

                    if cnt != 1 {
                        panic!(
                            "Read {} bytes from configured reader, expected exactly 1",
                            cnt
                        );
                    }

                    self.mem[self.dp] = buf[0];
                }
                '[' => {
                    if self.mem[self.dp] == 0 {
                        self.ip = *self
                            .loops
                            .get(&self.ip)
                            .expect("Unable to get matching bracket")
                    }
                }
                ']' => {
                    if self.mem[self.dp] != 0 {
                        self.ip = *self
                            .loops
                            .get(&self.ip)
                            .expect("Unable to get matching bracket")
                    }
                }
                c => panic!("Unexpected char in code: {}", c),
            };

            self.ip += 1;
        }

        self.mem
    }
}

impl<R> BrainfuckProgram<R, Stdout>
where
    R: Read,
{
    /// Create a new BrainfuckProgram, specifying the reader. The writer is assumed to be stdout
    pub fn new_with_reader(input: String, reader: R) -> Self {
        BrainfuckProgram::new_full(input, stdout(), reader)
    }
}

impl<W> BrainfuckProgram<Stdin, W>
where
    W: Write,
{
    /// Create a new BrainfuckProgram, specifying the writer. The reader is assumed to be stdin
    pub fn new_with_writer(input: String, writer: W) -> Self {
        BrainfuckProgram::new_full(input, writer, stdin())
    }
}

impl BrainfuckProgram<Stdin, Stdout> {
    /// Create a new BrainfuckProgram without specifying reader or writer. They are assumed to be
    /// stdin and stdout, respectively
    pub fn new(input: String) -> Self {
        BrainfuckProgram::new_full(input, stdout(), stdin())
    }
}

#[cfg(test)]
mod tests {
    use std::{array, io::Cursor, iter::repeat};

    use crate::arr;

    use super::*;

    #[test]
    fn bracket_matching() {
        let input = String::from("[[]][[]abc[]]");
        let prog = BrainfuckProgram::new(input);

        let mut expected_loops = HashMap::new();
        expected_loops.insert(1, 2);
        expected_loops.insert(0, 3);
        expected_loops.insert(5, 6);
        expected_loops.insert(4, 9);
        expected_loops.insert(7, 8);

        expected_loops.insert(2, 1);
        expected_loops.insert(3, 0);
        expected_loops.insert(6, 5);
        expected_loops.insert(9, 4);
        expected_loops.insert(8, 7);

        assert_eq!(prog.loops, expected_loops)
    }

    #[should_panic]
    #[test]
    fn mismatched_brackets() {
        let input = String::from("][");
        let _ = BrainfuckProgram::new(input);
    }

    #[should_panic]
    #[test]
    fn mismatched_brackets2() {
        let input = String::from("[[]");
        let _ = BrainfuckProgram::new(input);
    }

    #[test]
    fn addition() {
        let input1 = String::from("++");
        let prog1 = BrainfuckProgram::new(input1);

        let input2 = ['+'; 255].into_iter().collect();
        let prog2 = BrainfuckProgram::new(input2);

        let input3 = ['+'; 257].into_iter().collect();
        let prog3 = BrainfuckProgram::new(input3);

        let output1 = prog1.interpret_naive();
        let output2 = prog2.interpret_naive();
        let output3 = prog3.interpret_naive();

        let mut exp1 = [0u8; 30000];
        let mut exp2 = [0u8; 30000];
        let mut exp3 = [0u8; 30000];
        exp1[0] = 2;
        exp2[0] = 255;
        exp3[0] = 1;

        assert_eq!(output1, exp1);
        assert_eq!(output2, exp2);
        assert_eq!(output3, exp3);
    }

    #[test]
    fn subtraction() {
        let input1 = String::from("--");
        let input2 = ['-'; 255].into_iter().collect();
        let input3 = ['-'; 257].into_iter().collect();

        let prog1 = BrainfuckProgram::new(input1);
        let prog2 = BrainfuckProgram::new(input2);
        let prog3 = BrainfuckProgram::new(input3);

        let output1 = prog1.interpret_naive();
        let output2 = prog2.interpret_naive();
        let output3 = prog3.interpret_naive();

        let mut exp1 = [0u8; 30000];
        let mut exp2 = [0u8; 30000];
        let mut exp3 = [0u8; 30000];
        exp1[0] = 254;
        exp2[0] = 1;
        exp3[0] = 255;

        assert_eq!(output1, exp1);
        assert_eq!(output2, exp2);
        assert_eq!(output3, exp3);
    }

    #[test]
    fn inc_dp() {
        let input1 = String::from("+>++>+++>-");
        let mut input2: String = ['>'; 100].into_iter().collect();
        let mut input3: String = ['>'; 29999].into_iter().collect();
        input2.push('+');
        input3.push('+');

        let prog1 = BrainfuckProgram::new(input1);
        let prog2 = BrainfuckProgram::new(input2);
        let prog3 = BrainfuckProgram::new(input3);

        let output1 = prog1.interpret_naive();
        let output2 = prog2.interpret_naive();
        let output3 = prog3.interpret_naive();

        let exp1 = arr!([0; 30000], (1), (2), (3), (255));
        let exp2 = arr!([0; 30000], (0; 100), (1));
        let exp3 = arr![[0; 30000], (0; 29999), (1)];

        assert_eq!(output1, exp1);
        assert_eq!(output2, exp2);
        assert_eq!(output3, exp3);
    }

    #[test]
    fn dec_dp() {
        let input1 = String::from(">>>>>+<-<-<-<-");
        let input2 = String::from("<-<-<-<+");

        let prog1 = BrainfuckProgram::new(input1);
        let mut prog2 = BrainfuckProgram::new(input2);
        prog2.dp = 100;

        let output1 = prog1.interpret_naive();
        let output2 = prog2.interpret_naive();

        let exp1 = arr!([0; 30000], (0), (255; 4), (1));
        let exp2 = arr!([0; 30000], (0; 96), (1), (255; 3));

        assert_eq!(output1, exp1);
        assert_eq!(output2, exp2);
    }

    #[test]
    fn io() {
        let mut stdin_buf: Vec<u8> = (0..100).collect();
        let mut stdout_buf: Vec<u8> = vec![0; 100];

        let reader = Cursor::new(&mut stdin_buf);
        let writer = Cursor::new(&mut stdout_buf);

        let input = ",>".repeat(100) + "<.".repeat(100).as_str();

        let prog = BrainfuckProgram::new_full(input, writer, reader);
        let output = prog.interpret_naive();

        let exp = arr!([0u8; 30000]; 0..100);

        assert_eq!(output, exp);

        assert_eq!(stdin_buf, (0..100).collect::<Vec<_>>());
        assert_eq!(stdout_buf, (0..100).rev().collect::<Vec<_>>());
    }

    #[test]
    fn control_flow_basic() {
        // This program should print out every number between 1 and 255, then exit
        let input = String::from("+[+.]");

        let mut stdout_buf = [0u8; 1000];
        let writer = Cursor::new(&mut stdout_buf[..]);

        let prog = BrainfuckProgram::new_with_writer(input, writer);
        let output = prog.interpret_naive();

        assert_eq!(output, [0; 30000]);
        assert_eq!(stdout_buf, arr!([0u8; 1000]; 2..=255));
    }

    #[test]
    fn control_flow_extra() {
        // This program first enters the `[+.]` loop, during which it will print every number
        // from 2 to 255, then 0. It will then enter the `[-.]` loop, during which it will print
        // every number from 254 down to 0. It will then read user input and, if it is non-zero,
        // start over. This time will go the same way except the `[+.]` loop will print starting at
        // one more than the byte it reads from input instead of 2. It will continue in this
        // fashion until it recieves 0 as input, at which time it will increment the current memory
        // location and exit.
        let input = String::from("+[[+.]-[-.],]+");

        let stdin_buf = [1, 2, 3, 0];
        let mut stdout_buf = [0; 10000];

        let reader = Cursor::new(&stdin_buf[..]);
        let writer = Cursor::new(&mut stdout_buf[..]);

        let prog = BrainfuckProgram::new_full(input, writer, reader);
        let output = prog.interpret_naive();

        assert_eq!(output, arr!([0; 30000], (1)));

        let exp = arr!(
            [0; 10000];
            // In the first inner loop
            2..=255,
            [0],
            // In the second inner loop
            (0..255).rev(),
            // Here it reads 1 from user input then starts over, immediately entering loop 1
            2..=255,
            [0],
            // In the second loop again
            (0..255).rev(),
            // Here it reads 2 from user input then starts over, immediately entering loop 1
            3..=255,
            [0],
            // In the second loop again
            (0..255).rev(),
            // Here it reads 3 from user input then starts over, immediately entering loop 1
            4..=255,
            [0],
            // In the second loop again
            (0..255).rev()
            // Here it reads 0 from user input and exits, never to output again
        );

        assert_eq!(stdout_buf, exp);
    }
}
