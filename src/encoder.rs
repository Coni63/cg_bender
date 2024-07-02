use std::{cmp::min, collections::HashMap};

// fn find_substrings(s: &str) -> HashMap<&str, i32> {
//     let mut substr_count = HashMap::new();
//     let length = s.len();
//     for start in 0..length {
//         let offset = min(length, start + 10);
//         for end in start + 1..offset {
//             let substr = &s[start..end];
//             if !substr.chars().all(char::is_alphabetic) {
//                 continue;
//             }
//             *substr_count.entry(substr).or_insert(0) += 1;
//         }
//     }

//     substr_count
// }

// fn sort_substring(counter: HashMap<&str, i32>) -> Vec<(&str, i32)> {
//     let mut ans: Vec<(&str, i32)> = Vec::new();
//     for (subset, occurrences) in counter.iter() {
//         let gain = (subset.len() as i32) * (1 - occurrences) + 2;
//         if gain < 0 {
//             ans.push((*subset, gain));
//         }
//     }

//     ans.sort_by_key(|k| k.1);

//     ans
// }

// fn stringify(s: &str, macros: &[String]) -> String {
//     [s.to_string()]
//         .iter()
//         .chain(macros.iter())
//         .cloned()
//         .collect::<Vec<String>>()
//         .join(";")
// }

// fn compress_string(s: &str, macros: &mut Vec<String>, depth: i32, calls: &mut i32) -> String {
//     *calls += 1;
//     if depth >= 9 {
//         return stringify(s, macros);
//     }
//     let substr_count = find_substrings(s);
//     let sorted_count = sort_substring(substr_count);

//     if sorted_count.is_empty() {
//         return stringify(s, macros);
//     }

//     let mut shortest_solution = stringify(s, macros);
//     // Find the substring removing the most characters
//     for (subset, _) in sorted_count.iter().take(3) {
//         let s_subset = subset.to_string();
//         let mut copy_macros = macros.clone();
//         copy_macros.push(s_subset.clone());
//         let copy_s = s.replace(&s_subset, &depth.to_string());

//         let solution = compress_string(&copy_s, &mut copy_macros, depth + 1, calls);
//         if solution.len() < shortest_solution.len() {
//             shortest_solution = solution
//         }
//     }

//     shortest_solution
// }

// pub fn encode_actions(s: &str) -> String {
//     let mut calls = 0;
//     let mut macros: Vec<String> = vec![];
//     let ans = compress_string(s, &mut macros, 1, &mut calls);
//     eprintln!("Calls: {}", calls);
//     ans
// }

fn find_substrings<'a>(s: &'a str) -> HashMap<&'a str, i32> {
    let mut substr_count = HashMap::new();
    let length = s.len();
    for start in 0..length {
        let offset = min(length, start + 10);
        for end in start + 1..offset {
            let substr = &s[start..end];
            if substr.chars().all(char::is_alphabetic) {
                *substr_count.entry(substr).or_insert(0) += 1;
            }
        }
    }
    substr_count
}

fn sort_substring<'a>(counter: &HashMap<&'a str, i32>) -> Vec<(&'a str, i32)> {
    let mut ans: Vec<(&str, i32)> = counter
        .iter()
        .map(|(&subset, &occurrences)| {
            let gain = (subset.len() as i32) * (1 - occurrences) + 2;
            (subset, gain)
        })
        .filter(|&(_, gain)| gain < 0)
        .collect();

    ans.sort_by_key(|&(_, gain)| gain);
    ans
}

fn stringify(s: &str, macros: &[String]) -> String {
    [s.to_string()]
        .iter()
        .chain(macros.iter())
        .cloned()
        .collect::<Vec<String>>()
        .join(";")
}

fn compress_string(s: &str, macros: &mut Vec<String>, depth: i32, calls: &mut i32) -> String {
    *calls += 1;
    if depth >= 9 {
        return stringify(s, macros);
    }
    let substr_count = find_substrings(s);
    let sorted_count = sort_substring(&substr_count);

    if sorted_count.is_empty() {
        return stringify(s, macros);
    }

    let mut shortest_solution = stringify(s, macros);
    // Find the substring removing the most characters
    for (subset, _) in sorted_count.iter().take(3) {
        let s_subset = subset.to_string();
        macros.push(s_subset.clone());
        let copy_s = s.replace(&s_subset, &depth.to_string());

        let solution = compress_string(&copy_s, macros, depth + 1, calls);
        if solution.len() < shortest_solution.len() {
            shortest_solution = solution
        }
        macros.pop(); // Remove the last macro to avoid cloning
    }

    shortest_solution
}

pub fn encode_actions(s: &str) -> String {
    let mut calls = 0;
    let mut macros: Vec<String> = vec![];
    let ans = compress_string(s, &mut macros, 1, &mut calls);
    eprintln!("Calls: {}", calls);
    ans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding() {
        let s = String::from("DDDDRRDDDDDUUUUULLUUUULLLLUUDDRRRRDDDDRRDDDDDDDDDDDDLLUUUULLUUUUULURUULLLLRRRRDDLDRRRDDDDDLLLLUUUULRDDDDRRRRDDDDLLUDRRUUUULLUUUUULULLLLLUULLLLLLLLDDDDDDRRRRUDRRRRRRDLLDDDRRDDLLLLLLUUUULLLLUUUUUUUURRRRRRRRDRDRRRRRDRRDDDDDDDDDRRUUUUUUUUUUUULLUUUULLLLLLLLUUL");
        // let s = String::from("UUUUDDDDUUUUDDDD");
        let start_time = std::time::Instant::now();
        let ans = encode_actions(&s);
        assert!(!ans.is_empty());
        assert!(ans.len() < s.len());

        eprintln!("{:?}", ans);
        eprintln!("Time: {:?}", start_time.elapsed());
    }
}
