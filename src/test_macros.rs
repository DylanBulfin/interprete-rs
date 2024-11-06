//! Some macros to make testing easier (and to practice using macros)

/// This is a macro that makes it more convenient to create arrays. The format is hard to explain
/// but I hope that the examples will help to clarify
///
/// # Examples
/// ```
/// use interprete_rs::arr;
///
/// let arr1 = arr!([0; 10], (1), (2), (3; 5), (4), (5));
/// // 1, 2, 4, 5 specify singular elements and appear once in the output, (3; 5) specifies 5
/// // elements with value 3. The remaining value is the defined default (0) since only 9 values
/// // were specified
/// assert_eq!(arr1.len(), 10);
/// assert_eq!(arr1, [1, 2, 3, 3, 3, 3, 3, 4, 5, 0]);
///
/// let arr2 = arr!([5; 16], (10; 5), (-1; 10));
/// assert_eq!(arr2.len(), 16);
/// assert_eq!(arr2, [10, 10, 10, 10, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5]);
///
/// // The above both used the single element/range syntax, the following use the alternate syntax
/// // which acceps a list of iterators. Note the semicolon after the `[0; 100]` instead of a comma
/// // as in previous examples
/// let arr3 = arr!([0; 100]; (1..50), [1, 2]);
/// assert_eq!(arr3.len(), 100);
///
/// let mut expected3 = [0; 100];
/// for i in 0..49 {
///     expected3[i] = i + 1;
/// }
/// expected3[49] = 1;
/// expected3[50] = 2;
///
/// assert_eq!(arr3, expected3);
/// ```
#[macro_export]
macro_rules! arr {
    ( [$default:expr; $size:literal], $( ( $elem:expr $( ;$n:expr )?) ),* ) => {
        {
            let mut sum = 0;
            let mut vec = Vec::new();

            $(
                {
                    // For singular values (e.g. $n not defined), this evaluates to n = 1
                    // For ranges, this evaluates to n = 1 - 1 + $n
                    let n = 1 $(- 1 + $n)?;

                    sum += n;
                    for _ in 0..n {
                        vec.push($elem);
                    }
                }
            )*

            if sum > $size {
                panic!("Specified size not large enough to hold all data");
            }

            let mut arr = [$default; $size];

            for (i, v) in vec.into_iter().enumerate() {
                arr[i] = v;
            }

            arr
        }
    };
    ( [$default:expr; $size:literal]; $( $iter:expr ),* ) => {
        {
            let mut sum = 0;
            let mut vec = Vec::new();

            $(
                #[allow(for_loops_over_fallibles)]
                for v in $iter {
                    sum += 1;
                    vec.push(v);
                }
            )*

            if sum > $size {
                panic!("Specified size not large enough to hold all data");
            }

            let mut arr = [$default; $size];

            for (i, v) in vec.into_iter().enumerate() {
                arr[i] = v;
            }

            arr
        }
    }
}

/// This is an attempt at a nicer-looking `arr` macro that uses recursion. Macro recursion is not
/// optimized, so this may increase compile time vs. the other macro. This is specifically set up
/// to support literals and ranges. E.g. `arr_tt!([default; cnt], 1, (4; 3), 5)`.
macro_rules! arr_tt {
    () => {};
    ([ $default:expr; $cnt:literal ], $( $tail:tt)* ) => {
        {
            let mut sum = 0;
            let mut vec = Vec::new();

            arr_tt($($tail)*)
        }
    };
    ( $( $elem:expr ),+ ,$( $tail:tt )*) => {
        sum += 1;
        vec.push($elem);
        
        arr_tt!($($tail)*)
    }
}

/// This is a macro to allow defining HashMaps in a similar way to the `vec!` macro. I use
/// python-ish syntax but with comma-separated pairs since colons can't be used as literals in a
/// rust macro pattern definition
///
/// # Examples
/// ```
/// use interprete_rs::map;
///
/// let map = map!{(1, 2), (2, 3), (4, 3), (5, 2)};
///
/// let mut keys: Vec<_> = map.keys().collect();
/// let mut vals: Vec<_> = map.values().collect();
/// keys.sort();
/// vals.sort();
/// assert_eq!(keys, vec![&1, &2, &4, &5]);
/// assert_eq!(vals, vec![&2, &2, &3, &3]);
///
/// assert_eq!(map.get(&1), Some(&2));
/// assert_eq!(map.get(&2), Some(&3));
/// assert_eq!(map.get(&4), Some(&3));
/// assert_eq!(map.get(&5), Some(&2));
/// assert_eq!(map.get(&3), None);
/// ```
#[macro_export]
macro_rules! map {
    { $( ( $key:expr, $val:expr ) ),+ } => {
        {
            let mut map = std::collections::HashMap::new();

            $(
                map.insert($key, $val);
            )+

            map
        }
    };
}

/// Haskell-inspired list comprehension
///
/// # Examples
/// ```
/// use interprete_rs::list_comp;
///
/// let l1 = list_comp!(a * 2; [1, 2, 3] => a);
/// assert_eq!(l1, [2, 4, 6]);
///
/// let l2 = list_comp!(a.to_ascii_lowercase(); ["ABC", "BCD", "ADE😵"] => a; a.is_ascii());
/// assert_eq!(l2, ["abc", "bcd"]);
///
/// let l3 = list_comp!(a * 2; 0..1000 => a);
/// let l4 = list_comp!(a; 0..2000 => a; a % 2 == 0);
/// assert_eq!(l3, l4);
/// ```
#[macro_export]
macro_rules! list_comp {
    [ $func:expr; $lst:expr => $var:ident $( ;$cond:expr )? ] => {
        {
            let mut vec = Vec::new();

            for $var in $lst {
                $(if !$cond {continue;})?

                vec.push($func);
            }

            vec
        }
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    use std::collections::HashMap;

    #[test]
    fn arr_macro_ranges() {
        let arr = arr!([0; 30000], (1), (2), (3; 10), (7));

        let mut expected = [0; 30000];
        expected[0] = 1;
        expected[1] = 2;
        (2..12).for_each(|i| expected[i] = 3);
        expected[12] = 7;

        assert_eq!(arr, expected);
    }

    #[test]
    fn arr_macro_iters() {
        // Note the semicolon following the [0; 100]
        let arr = arr!([0; 100]; 1..50, [1, 2], Some(4));

        let mut expected = [0u32; 100];
        expected
            .iter_mut()
            .take(49)
            .enumerate()
            .for_each(|(i, v)| *v = i as u32 + 1);
        expected[49] = 1;
        expected[50] = 2;
        expected[51] = 4;

        assert_eq!(arr, expected);
    }

    #[test]
    fn map_macro() {
        let map = map! {(1, 2), (2, 3), (3, 4), (5, 4)};

        let mut keys: Vec<_> = map.keys().collect();
        let mut vals: Vec<_> = map.values().collect();
        keys.sort();
        vals.sort();

        assert_eq!(keys, vec![&1, &2, &3, &5]);
        assert_eq!(vals, vec![&2, &3, &4, &4]);

        let mut expected = HashMap::new();
        expected.insert(1, 2);
        expected.insert(2, 3);
        expected.insert(3, 4);
        expected.insert(5, 4);

        assert_eq!(map, expected);
    }

    #[test]
    fn list_comp() {
        let comp1 = list_comp!(a * 2; [1, 2, 3] => a);
        let comp2 = list_comp!(a.is_ascii(); ["ABC", "BCD", "😀"] => a);
        let comp3 = list_comp!(a + 5; [1, 2, 3, 4, 5] => a; a < 4);
        let comp4 = list_comp!(a / 10; 0..100 => a; a % 2 == 0);

        assert_eq!(comp1, vec![2, 4, 6]);
        assert_eq!(comp2, vec![true, true, false]);
        assert_eq!(comp3, vec![6, 7, 8]);
        assert_eq!(comp4.len(), 50);
        assert_eq!(
            comp4,
            arr!([0; 50], (0; 5), (1; 5), (2; 5), (3; 5), (4; 5), (5; 5), (6; 5), (7; 5), (8; 5), (9; 5))
        );
    }
}
