//! Some macros to make testing easier (and to practice using macros)

/// This is a macro that makes it more convenient to create arrays. The format is the base array
/// args (default value and size) followed by a variable number of arguments, which can be either a
/// single element or an element with an associated count. The examples below will hopefully make
/// things clear.
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
/// ```
#[macro_export]
macro_rules! arr {
    [ [$default:expr; $size:literal], $( ( $elem:expr $( ;$n:expr )?) ),+  ] => {
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
            )+

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

#[macro_export]
macro_rules! list_comp {
    [ $func:expr; $lst:expr => $var:ident $( ;$cond:expr )? ] => {
        {
            let mut vec = Vec::new();

            for $var in $lst.iter() {
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
    fn arr_macro() {
        let arr = arr!([0; 30000], (1), (2), (3; 10), (7));

        let mut expected = [0; 30000];
        expected[0] = 1;
        expected[1] = 2;
        (2..12).for_each(|i| expected[i] = 3);
        expected[12] = 7;

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

        assert_eq!(comp1, vec![2, 4, 6]);
        assert_eq!(comp2, vec![true, true, false]);
    }
}
