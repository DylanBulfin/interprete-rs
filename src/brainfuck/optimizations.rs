//! This module contains various optimization strategies for brainfuck programs.

use std::cmp::Ordering;

//const MACROINSTRUCTION_CHARS: [char; 22] = [
//    '(', ')', ';', '*', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'p', 'm', 'f', 'b', 'i',
//    'o', '{', '}',
//];

fn to_alt_opcode(c: char) -> char {
    match c {
        '+' => 'p',
        '-' => 'm',
        '<' => 'b',
        '>' => 'f',
        '.' => 'o',
        ',' => 'i',
        '[' => '{',
        ']' => '}',
        _ => panic!("Unexpected char: {}", c),
    }
}

/// Macro to allow simpler implementation of full pair reductions.
///
/// # Examples
/// ```
/// use interprete_rs::max_reduction;
/// let input: Vec<char> = "-++++--+".chars().collect();
/// max_reduction!('+', '-', input);
/// ```
#[macro_export]
macro_rules! max_reduction {
    ( $c1:literal, $c2:literal, $input:ident ) => {{
        let mut res = Vec::new();

        let mut c1_count = 0;
        let mut c2_count = 0;

        let handle_seq = |c1c: usize, c2c: usize, r: &mut Vec<char>| match c1c.cmp(&c2c) {
            std::cmp::Ordering::Less => r.append(&mut vec![$c2; c2c - c1c]),
            std::cmp::Ordering::Greater => r.append(&mut vec![$c1; c1c - c2c]),
            _ => (),
        };

        // First we want to identify contiguous sequences of `+` and `-`
        for c in $input {
            if c == $c1 {
                // Part of the current sequence, add it
                c1_count += 1;
            } else if c == $c2 {
                c2_count += 1;
            } else {
                handle_seq(c1_count, c2_count, &mut res);

                (c1_count, c2_count) = (0, 0);

                res.push(c)
            }
        }

        // Handle any leftover sequence
        handle_seq(c1_count, c2_count, &mut res);

        res
    }};
}

/// Since we use wrapping arithmetic we can reduce any contiguous sequence of math operations which
/// contain at least one `-` and one `+`
pub fn math_reduction(input: Vec<char>) -> Vec<char> {
    max_reduction!('+', '-', input)
}

/// The math functions can be reduced fully without loss of correctness, but a full reduction of
/// cell move operations (`<` and `>`) can cause differences in behavior which is unwanted. Namely
/// `<>` when at memory location 0 or `><` when at memory location 29999 should crash, but a naive
/// reduction would reduce it to nothing. This optimization will need the user to set a high
/// optimization level
pub fn full_dp_reduction(input: Vec<char>) -> Vec<char> {
    max_reduction!('>', '<', input)
}

/// Alternative for always-correct reductions: reduce a sequence of `<` and `>` to a
/// tuple that represents the range of actual values relative to starting location. E.g.
/// `>>>>><<<<<<` would reduce to `(5;1)<`. This indicates that the memory pointer went a max of
/// 5 cells forward from the starting location, and 1 cell backward. That way the interpreter can
/// check if the operation would have caused an overflow. Then afterwards we insert `<` or `>` as
/// in a full reduction and allow the interpreter to handle them normally
pub fn safe_dp_reduction(input: Vec<char>) -> Vec<char> {
    let mut res = Vec::new();

    let mut curr_diff = 0i32;
    let mut max_left = 0u32;
    let mut max_right = 0u32;

    let handle_seq = |cd: i32, ml: u32, mr: u32, r: &mut Vec<char>| {
        if ml == 0 && cd >= mr as i32 || mr == 0 && cd <= -(ml as i32) {
            // No need for bounds check since current position is the most extreme point reached
        } else {
            // Add macro-instruction to output
            r.append(&mut format!("({};{})", ml, mr).chars().collect());
        }

        match cd.cmp(&0) {
            Ordering::Less => r.append(&mut vec!['<'; cd.unsigned_abs() as usize]),
            Ordering::Greater => r.append(&mut vec!['>'; cd as usize]),
            _ => (),
        }
    };

    for c in input {
        if c == '<' {
            curr_diff -= 1;
            if curr_diff < -(max_left as i32) {
                max_left += 1;
            }
        } else if c == '>' {
            curr_diff += 1;
            if curr_diff > max_right as i32 {
                max_right += 1;
            }
        } else {
            handle_seq(curr_diff, max_left, max_right, &mut res);
            (max_left, max_right, curr_diff) = (0, 0, 0);

            res.push(c);
        }
    }

    handle_seq(curr_diff, max_left, max_right, &mut res);

    res
}

/// This optimization reduces long strings of identical instructions to a single macro-instruction.
/// For example, if it recieves the input `>>>>>` it would reduce to `(5*f)`
pub fn compress_seq(input: Vec<char>) -> Vec<char> {
    let compressable = ['+', '-', '<', '>'];

    let mut res = Vec::new();

    let mut curr_char = 0 as char;
    let mut count = 0;

    let handle_seq = |cc: char, cnt: u32, r: &mut Vec<char>| match cnt.cmp(&1) {
        Ordering::Less => (),
        Ordering::Equal => r.push(cc),
        Ordering::Greater => {
            r.append(&mut format!("({}*{})", cnt, to_alt_opcode(cc)).chars().collect())
        }
    };

    for c in input {
        if compressable.contains(&c) && c == curr_char {
            count += 1;
        } else {
            handle_seq(curr_char, count, &mut res);

            curr_char = c;
            count = 1;
        }
    }

    handle_seq(curr_char, count, &mut res);

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! mk_test {
        ([$($input:expr)*], [$($output:expr)*], $func:ident) => {
            $(
                {
                    let input = $input;
                    let output = $output;

                    assert_eq!(
                        $func(input.chars().collect()),
                        output.chars().collect::<Vec<_>>()
                    );
                }
            )*
        };
    }

    #[test]
    fn math_reduction_test() {
        let input1 = "+-+-+----+-+-++-+"; // Should be reduced to '-'
        let input2 = "----+"; // Should reduce to '---'
        let input3 = "+++++++"; // Should stay the same
        let input4 = "[+-++----+]>>-+++-"; // Should reduce to "[-]>>+"

        let output1 = "-";
        let output2 = "---";
        let output3 = "+++++++";
        let output4 = "[-]>>+";

        assert_eq!(
            math_reduction(input1.chars().collect()),
            output1.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            math_reduction(input2.chars().collect()),
            output2.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            math_reduction(input3.chars().collect()),
            output3.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            math_reduction(input4.chars().collect()),
            output4.chars().collect::<Vec<_>>()
        );
    }

    #[test]
    fn full_dp_reduction_test() {
        let input1 = "><><><<<<><><>><>"; // Should be reduced to '<'
        let input2 = "<<<<>"; // Should reduce to '<<<'
        let input3 = ">>>>>>>"; // Should stay the same
        let input4 = "[><>><<<<>]>><>>><"; // Should reduce to "[<]>>>"

        let output1 = "<";
        let output2 = "<<<";
        let output3 = ">>>>>>>";
        let output4 = "[<]>>>";

        assert_eq!(
            full_dp_reduction(input1.chars().collect()),
            output1.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            full_dp_reduction(input2.chars().collect()),
            output2.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            full_dp_reduction(input3.chars().collect()),
            output3.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            full_dp_reduction(input4.chars().collect()),
            output4.chars().collect::<Vec<_>>()
        );
    }

    #[test]
    fn safe_dp_reduction_test() {
        let input1 = "><><><<<<><><>><>"; // Should be reduced to '(3;1)<'
        let input2 = "<<<<>"; // Should reduce to '(4;0)<<<'
        let input3 = ">>>>>>>"; // Should stay the same
        let input4 = "[><>><<<<>]>><>>><"; // Should reduce to "[(2,2)<](0;4)>>>"

        let output1 = "(3;1)<";
        let output2 = "(4;0)<<<";
        let output3 = ">>>>>>>";
        let output4 = "[(2;2)<](0;4)>>>";

        assert_eq!(
            safe_dp_reduction(input1.chars().collect()),
            output1.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            safe_dp_reduction(input2.chars().collect()),
            output2.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            safe_dp_reduction(input3.chars().collect()),
            output3.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            safe_dp_reduction(input4.chars().collect()),
            output4.chars().collect::<Vec<_>>()
        );
    }

    #[test]
    fn compress_seq_test() {
        let input1 = "++++++++<++++";
        let input2 = "<<<<<<<->>>>>>";
        let input3 = "(13;2)<<<<<<<<<<<<";
        let input4 = "[(13;2)<<<<<]+<<<<<<<";

        let output1 = "(8*p)<(4*p)";
        let output2 = "(7*b)-(6*f)";
        let output3 = "(13;2)(12*b)";
        let output4 = "[(13;2)(5*b)]+(7*b)";

        assert_eq!(
            compress_seq(input1.chars().collect()),
            output1.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            compress_seq(input2.chars().collect()),
            output2.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            compress_seq(input3.chars().collect()),
            output3.chars().collect::<Vec<_>>()
        );
        assert_eq!(
            compress_seq(input4.chars().collect()),
            output4.chars().collect::<Vec<_>>()
        );
    }
}
