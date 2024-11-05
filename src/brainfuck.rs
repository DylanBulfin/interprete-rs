use std::collections::HashMap;

pub const VALID_CHARS: [char; 8] = ['<', '>', '+', '-', '.', ',', '[', ']'];

pub struct BrainfuckProgram {
    code: Vec<char>,
    mem: [u8; 30000],
    ip: usize,
    dp: usize,
    loops: HashMap<usize, usize>, // Matching pairs of brackets
}

impl BrainfuckProgram {
    pub fn new(input: String) -> Self {
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
            mem: [0; 30000],
            ip: 0,
            dp: 0,
        }
    }
}

pub fn naive_interpreter(mut prog: BrainfuckProgram) -> [u8; 30000] {
    loop {
        if prog.ip >= prog.code.len() {
            // Reached end of program
            break;
        }
        match prog.code[prog.ip] {
            '<' => {
                prog.dp = prog
                    .dp
                    .checked_sub(1)
                    .expect("Data pointer is 0, cannot decrement")
            }
            '>' => {
                if prog.dp < 29999 {
                    prog.dp += 1
                } else {
                    panic!("Data pointer is 29999, cannot increment")
                }
            }
            '+' => prog.mem[prog.dp] = prog.mem[prog.dp].wrapping_add(1),
            '-' => prog.mem[prog.dp] = prog.mem[prog.dp].wrapping_sub(1),
            '.' => print!("{}", prog.mem[prog.dp] as char),
            ',' => unimplemented!(), // Keyboard input is annoying so I may do file input
            '[' => {
                if prog.mem[prog.dp] == 0 {
                    prog.ip = *prog
                        .loops
                        .get(&prog.ip)
                        .expect("Unable to get matching bracket")
                }
            }
            ']' => {
                if prog.mem[prog.dp] != 0 {
                    prog.ip = *prog
                        .loops
                        .get(&prog.ip)
                        .expect("Unable to get matching bracket")
                }
            }
            c => panic!("Unexpected char in code: {}", c),
        };

        prog.ip += 1;
    }

    prog.mem
}

#[cfg(test)]
mod tests {
    use std::array;

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

        let output1 = naive_interpreter(prog1);
        let output2 = naive_interpreter(prog2);
        let output3 = naive_interpreter(prog3);

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

        let output1 = naive_interpreter(prog1);
        let output2 = naive_interpreter(prog2);
        let output3 = naive_interpreter(prog3);

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

        let output1 = naive_interpreter(prog1);
        let output2 = naive_interpreter(prog2);
        let output3 = naive_interpreter(prog3);

        //let mut exp1 = [0u8; 30000];
        //let mut exp2 = [0u8; 30000];
        //let mut exp3 = [0u8; 30000];
        //exp1[0] = 1;
        //exp1[1] = 2;
        //exp1[2] = 3;
        //exp1[3] = 255;
        //exp2[100] = 1;
        //exp3[29999] = 1;
        //
        let exp1 = arr!([0; 30000], (1; 1), (2; 1), (3; 1), (255; 1), (0; 29996));
        let exp2 = arr!([0; 30000], (0; 100), (1; 1), (0; 29899));
        let exp3 = arr![[0; 30000], (0; 29999), (1; 1)];

        assert_eq!(output1, exp1);
        assert_eq!(output2, exp2);
        assert_eq!(output3, exp3);
    }
}
