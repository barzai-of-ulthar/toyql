/// This file contains utilities to support tests, particularly extensional tests that require
/// sources of large number of arbitrary values.

use rand::{distr::StandardUniform, rngs::SmallRng, Rng, SeedableRng};

/// Generate `how_many` signed integers.  Special values (e.g. 1, 0, max) will appear
/// early in the list.
pub fn example_ints(how_many: usize) -> Vec<i64> {
    let mut how_many = how_many;
    let basic_examples: Vec<i64> = vec![1, 0, i64::MAX, -1, i64::MIN, 100, 1000, -100];
    let mut result: Vec<i64> = basic_examples.iter().take(how_many).cloned().collect();
    how_many -= result.len();
    if how_many > 0 {
        let rng = SmallRng::seed_from_u64(42);
        result.extend(rng.random_iter::<i64>().take(how_many));
    }
    result
}

/// Generate `how_many` floats.  Special values (e.g. 1, 0, max) will appear
/// early in the list.
pub fn example_floats(how_many: usize) -> Vec<f64> {
    let mut how_many = how_many;
    let basic_examples: Vec<f64> = vec![
        1.0, 0.0, -1.0, 1e14, 1e-14, -0.0,
        f64::MAX, f64::MIN_POSITIVE, f64::MIN, -f64::MIN_POSITIVE,
        f64::INFINITY, f64::NEG_INFINITY, f64::NAN,
        /* TODO:  Non-canonical NAN values */];
    let mut result: Vec<f64> = basic_examples.iter().take(how_many).cloned().collect();
    how_many -= result.len();
    if how_many > 0 {
        let rng = SmallRng::seed_from_u64(42);
        result.extend(rng.random_iter::<f64>().take(how_many));
    }
    result
}

/// Generate `how_many` strings.  Special values (e.g. empty string, escapes) will appear
/// early in the list.
pub fn example_strings(how_many: usize) -> Vec<String> {
    let mut how_many = how_many;
    let basic_examples: Vec<&str> = vec![
        "abc", "a", "100",
        " \\ ",
        "\'", "\n", "\\",
        "\\\\", "\\\\\\",
        "\"", 
        "",
    ];
    let mut result: Vec<String> = basic_examples.iter().take(how_many)
        .map(|x| str::to_string(x)).collect();
    how_many -= result.len();
    let mut rng = SmallRng::seed_from_u64(42);
    for _ in 0..how_many {
        let len: usize = rng.random_range(1..10);
        let chars_to_add: Vec<char> = rng.clone().sample_iter(StandardUniform).take(len).collect();
        result.push(chars_to_add.into_iter().collect());
    };
    result
}
