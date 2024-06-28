use std::{cmp::min, collections::HashMap};

fn find_substrings(s: &str) -> HashMap<String, i32> {
    let mut substr_count = HashMap::new();
    let length = s.len();
    for start in 0..length {
        let end = min(length, start + 12);
        for end in (start + 2)..end {
            // take substrings of length 2 or more
            let substr: String = s[start..end].to_string();
            if !substr.chars().all(char::is_alphabetic) {
                continue;
            }
            *substr_count.entry(substr).or_insert(0) += 1;
        }
    }

    substr_count
}

fn gain(subset: &str, occurrences: i32) -> i32 {
    (subset.len() as i32) * (1 - occurrences) + 2
}

fn stringify(s: &str, macros: &[String]) -> String {
    [s.to_string()]
        .iter()
        .chain(macros.iter())
        .cloned()
        .collect::<Vec<String>>()
        .join(";")
}

fn compress_string(s: &str, macros: &mut Vec<String>) -> String {
    if macros.len() == 9 {
        return stringify(s, macros);
    }

    let mut substr_count = find_substrings(s);

    // Remove substrings that appear only once
    substr_count.retain(|_, &mut v| v > 1);

    // Find the substring removing the most characters
    let mut winner = String::new();
    let mut g = 0;

    for (k, v) in substr_count.iter() {
        let _g = gain(k, *v);
        if _g < g {
            g = _g;
            winner = k.clone();
        }
    }

    if winner.is_empty() {
        return stringify(s, macros);
    }

    macros.push(winner.clone());
    let s_macro = macros.len().to_string();
    let s = s.replace(&winner, &s_macro);

    compress_string(&s, macros)
}

pub fn encode_actions(s: &str) -> String {
    let mut macros: Vec<String> = vec![];
    compress_string(s, &mut macros)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding() {
        let s = String::from("DDDDRRDDDDDUUUUULLUUUULLLLUUDDRRRRDDDDRRDDDDDDDDDDDDLLUUUULLUUUUULURUULLLLRRRRDDLDRRRDDDDDLLLLUUUULRDDDDRRRRDDDDLLUDRRUUUULLUUUUULULLLLLUULLLLLLLLDDDDDDRRRRUDRRRRRRDLLDDDRRDDLLLLLLUUUULLLLUUUUUUUURRRRRRRRDRDRRRRRDRRDDDDDDDDDRRUUUUUUUUUUUULLUUUULLLLLLLLUUL");
        // let s = String::from("UUUUDDDDUUUUDDDD");
        let ans = encode_actions(&s);
        assert!(!ans.is_empty());
        assert!(ans.len() < s.len());

        eprintln!("{:?}", ans);
    }
}
