use regex::Regex;
use std::convert::TryInto;

// not sure what to do about ゎ (small wa)
const SMALL_HIRAGANA: [char; 8] = ['ゃ', 'ゅ', 'ょ', 'ぁ', 'ぃ', 'ぅ', 'ぇ', 'ぉ'];

fn is_normal_hiragana(c: &char) -> bool {
    !SMALL_HIRAGANA.contains(&c)
}

fn mora_len(hiragana_word: &str) -> usize {
    let len = hiragana_word
        .chars()
        .filter(is_normal_hiragana)
        .collect::<Vec<char>>()
        .len();

    len
}

fn split_to_mora(hiragana_word: &str) -> Vec<String> {
    let v: Vec<char> = hiragana_word.chars().collect();
    let mut ret_vec: Vec<String> = Vec::new();
    for (i, x) in v.clone().iter().enumerate() {
        if is_normal_hiragana(x) {
            ret_vec.push(format!("{}", x));
        } else {
            ret_vec.pop();
            ret_vec.push(format!("{}{}", v[i - 1], x));
        }
    }

    ret_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_hiragana_filter() {
        assert_eq!(is_normal_hiragana(&'ぁ'), false);
        assert_eq!(is_normal_hiragana(&'ゅ'), false);
        assert_eq!(is_normal_hiragana(&'ゃ'), false);
        assert_eq!(is_normal_hiragana(&'っ'), true);
        assert_eq!(is_normal_hiragana(&'し'), true);
        assert_eq!(is_normal_hiragana(&'あ'), true);
        assert_eq!(is_normal_hiragana(&'わ'), true);
        assert_eq!(is_normal_hiragana(&'つ'), true);
        assert_eq!(is_normal_hiragana(&'ゎ'), true);
    }

    #[test]
    fn test_mora_no_small_hiragana() {
        assert_eq!(mora_len("てんか"), 3);
        assert_eq!(mora_len("げつようび"), 5);
    }

    #[test]
    fn test_mora_contain_small_hiragana() {
        assert_eq!(mora_len("きょうみ"), 3);
        assert_eq!(mora_len("じょうきょう"), 4);
        assert_eq!(mora_len("しんぎょうそう"), 6);
    }

    #[test]
    fn test_split_mora() {
        assert_eq!(split_to_mora("きょうみ"), vec!["きょ", "う", "み"]);
        assert_eq!(
            split_to_mora("じょうきょう"),
            vec!["じょ", "う", "きょ", "う"]
        );
        assert_eq!(
            split_to_mora("しんぎょうそう"),
            vec!["し", "ん", "ぎょ", "う", "そ", "う"]
        );
    }
}
