//! Identifier subword splitting: `fetchUserByID` and `fetch_user_by_id` -> [fetch, user, by, id].

pub fn split_identifier(name: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_alphanumeric() {
            flush(&mut cur, &mut parts);
            continue;
        }
        let boundary = c.is_uppercase()
            && i > 0
            && (chars[i - 1].is_lowercase()
                || chars[i - 1].is_ascii_digit()
                || (chars[i - 1].is_uppercase()
                    && chars.get(i + 1).is_some_and(|n| n.is_lowercase())));
        if boundary {
            flush(&mut cur, &mut parts);
        }
        cur.push(c.to_ascii_lowercase());
    }
    flush(&mut cur, &mut parts);
    parts
}

fn flush(cur: &mut String, parts: &mut Vec<String>) {
    if !cur.is_empty() {
        parts.push(std::mem::take(cur));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_camel_snake_and_acronyms() {
        assert_eq!(split_identifier("fetchUserById"), ["fetch", "user", "by", "id"]);
        assert_eq!(split_identifier("fetch_user_by_id"), ["fetch", "user", "by", "id"]);
        assert_eq!(split_identifier("parseHTTPResponse"), ["parse", "http", "response"]);
        assert_eq!(split_identifier("_private"), ["private"]);
    }
}
