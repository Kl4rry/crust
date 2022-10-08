pub fn levenshtein_stripped(lhs: &str, rhs: &str) -> usize {
    let lhs = lhs.trim().to_lowercase();
    let rhs = rhs.trim().to_lowercase();
    distance::damerau_levenshtein(no_ext(&lhs), no_ext(&rhs))
}

fn no_ext(input: &str) -> &str {
    let pos = memchr::memchr(b'.', input.as_bytes());
    match pos {
        Some(pos) if pos > 0 => {
            let len = input.len();
            &input[..len - (len - pos)]
        }
        _ => input,
    }
}
