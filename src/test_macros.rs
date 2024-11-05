/// Some macros to make testing easier (and to practice using macros)

#[macro_export]
macro_rules! arr {
    ( $c: expr, $( ($elem:expr; $n:expr) ),+ ) => {
        {
            let mut sum = 0;
            let mut vec = Vec::new();

            $(
                sum += $n;
                for _ in 0..$n {
                    vec.push($elem);
                }
            )+

            if sum > $c {
                panic!("Given total count is smaller than sum of given segments' size")
            }
            if sum == 0 {
                panic!("arr! macro unable to handle 0-length arrays")
            }

            let mut arr = [vec[0]; $c];

            for (i, v) in vec.into_iter().enumerate() {
                arr[i] = v;
            }

            arr
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arr_macro() {
        let arr = arr![10, (3; 5), (4; 5)];

        assert_eq!(arr, [3, 3, 3, 3, 3, 4, 4, 4, 4, 4])
    }
}
