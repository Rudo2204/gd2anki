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

pub fn get_full_nhk_accent<'a>(hiragana_word: &'a str, raw_nhk_accent: &'a str) -> String {
    let full_accent = format!(
        "{}{}",
        std::iter::repeat("0")
            .take(hiragana_count_char(hiragana_word) - raw_nhk_accent.len())
            .collect::<String>(),
        raw_nhk_accent
    );
    full_accent
}

fn hiragana_count_char(hiragana: &str) -> usize {
    hiragana.chars().collect::<Vec<char>>().len()
}

fn make_single_accent(hiragana: &str, mora_pos: u8, pitch_number: u8, num_chars: u8) -> String {
    let ret_string: String = if pitch_number == 1 {
        // atamadaka
        if mora_pos == 1 {
            if hiragana_count_char(hiragana) == 2 {
                "12".to_string()
            } else {
                "2".to_string()
            }
        } else {
            std::iter::repeat("0")
                .take(hiragana_count_char(hiragana))
                .collect::<String>()
        }
    } else if pitch_number == 0 {
        // heiban
        if mora_pos == 1 {
            std::iter::repeat("0")
                .take(hiragana_count_char(hiragana))
                .collect::<String>()
        } else {
            std::iter::repeat("1")
                .take(hiragana_count_char(hiragana))
                .collect::<String>()
        }
    } else if pitch_number == num_chars {
        // odaka
        if mora_pos == 1 {
            "0".to_string()
        } else if mora_pos < pitch_number {
            std::iter::repeat("1")
                .take(hiragana_count_char(hiragana))
                .collect::<String>()
        } else {
            "2".to_string()
        }
    } else {
        // nakadaka
        if mora_pos == 1 || mora_pos > pitch_number {
            std::iter::repeat("0")
                .take(hiragana_count_char(hiragana))
                .collect::<String>()
        } else if mora_pos < pitch_number {
            std::iter::repeat("1")
                .take(hiragana_count_char(hiragana))
                .collect::<String>()
        } else {
            if hiragana_count_char(hiragana) == 2 {
                "12".to_string()
            } else {
                "2".to_string()
            }
        }
    };

    ret_string
}

fn generate_nhk_accent_from_pitch_number(hiragana_word: &str, pitch_number: u8) -> String {
    let num_chars = hiragana_word.chars().collect::<Vec<char>>().len();
    let mora_vec = split_to_mora(hiragana_word);

    let mut ret_string: String = "".to_owned();
    for (i, x) in mora_vec.iter().enumerate() {
        ret_string.push_str(&make_single_accent(
            x,
            (i + 1).try_into().unwrap(),
            pitch_number,
            num_chars.try_into().unwrap(),
        ));
    }

    ret_string
}

#[derive(Debug, PartialEq)]
struct HiraNhkPitch {
    hiragana: String,
    nhk_accent: String,
}

fn generate_nhk_accent_from_pitch_number_string<'a>(
    hiragana: &str,
    pitch_string: &str,
    hatsuon: &str,
) -> HiraNhkPitch {
    let re = Regex::new(r"(\[Akz\]|[ぁ-ん])").unwrap();
    //let caps = re.captures_iter(hatsuon);
    let hatsu_clean: String = re.find_iter(hatsuon).map(|cap| cap.as_str()).collect();
    let hira_v: Vec<&str> = hatsu_clean.split("[Akz]").collect();
    let pitch_v: Vec<u8> = pitch_string
        .split("-")
        .map(|num| {
            num.parse::<u8>()
                .expect("could not parse pitch string to vec u8")
        })
        .collect();

    let hira_ret: String = hira_v.clone().into_iter().collect();
    if hira_ret != hiragana {
        panic!("hiragana = {} but hatsu_clean = {}", hiragana, hatsu_clean);
    }
    let mut ret_nhk_accent = String::new();
    for (i, x) in hira_v.iter().enumerate() {
        ret_nhk_accent.push_str(&generate_nhk_accent_from_pitch_number(&x, pitch_v[i]));
    }
    return HiraNhkPitch {
        hiragana: hira_ret,
        nhk_accent: ret_nhk_accent,
    };
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
    fn test_split_mora_norm() {
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

    #[test]
    fn test_split_mora_double_small() {
        // NOTE: for mora small tsu っ is considered a special mora by itself
        // but when we split to syllables it would look like => `しゃっ きん`
        assert_eq!(split_to_mora("しゃっきん"), vec!["しゃ", "っ", "き", "ん"]);
        assert_eq!(split_to_mora("しゅっせき"), vec!["しゅ", "っ", "せ", "き"]);
        assert_eq!(split_to_mora("しゅっさん"), vec!["しゅ", "っ", "さ", "ん"]);
    }

    #[test]
    fn test_get_full_nhk_accent() {
        assert_eq!(get_full_nhk_accent("あひきやうげん", "112000"), "0112000");
        assert_eq!(get_full_nhk_accent("やちん", "200"), "200");
        assert_eq!(get_full_nhk_accent("きょうみ", "1200"), "1200");
        assert_eq!(get_full_nhk_accent("じょうきょう", "1111"), "001111");
        assert_eq!(get_full_nhk_accent("さむけ", "12"), "012");
        assert_eq!(get_full_nhk_accent("かんき", "200"), "200");
        assert_eq!(get_full_nhk_accent("たてもの", "200"), "0200");
        assert_eq!(get_full_nhk_accent("たてもの", "012"), "0012");
        assert_eq!(get_full_nhk_accent("しゃっきん", "120"), "00120");
    }

    // nhk pitch generation tests
    // 頭高 - high low tests
    #[test]
    fn atamadaka_case1_old() {
        // やちん - 200
        assert_eq!(make_single_accent("や", 1, 1, 3), "2");
        assert_eq!(make_single_accent("ち", 2, 1, 3), "0");
        assert_eq!(make_single_accent("ん", 3, 1, 3), "0");
    }

    #[test]
    fn atamadaka_case4_old() {
        // きょうみ - 1200
        assert_eq!(make_single_accent("きょ", 1, 1, 3), "12");
        assert_eq!(make_single_accent("う", 2, 1, 3), "0");
        assert_eq!(make_single_accent("み", 3, 1, 3), "0");
    }

    #[test]
    fn atamadaka_case1_new() {
        // あいさつ - 2000
        assert_eq!(make_single_accent("あ", 1, 1, 4), "2");
        assert_eq!(make_single_accent("い", 2, 1, 4), "0");
        assert_eq!(make_single_accent("さ", 3, 1, 4), "0");
        assert_eq!(make_single_accent("つ", 4, 1, 4), "0");
    }

    #[test]
    fn atamadaka_case2_new() {
        // でんき - 200
        assert_eq!(make_single_accent("で", 1, 1, 3), "2");
        assert_eq!(make_single_accent("ん", 2, 1, 3), "0");
        assert_eq!(make_single_accent("き", 3, 1, 3), "0");
    }

    #[test]
    fn atamadaka_case3_new() {
        // あき (秋) - 20
        assert_eq!(make_single_accent("あ", 1, 1, 3), "2");
        assert_eq!(make_single_accent("き", 2, 1, 3), "0");
    }

    // 平板 - low high tests
    #[test]
    fn heiban_case2_old() {
        // じょうきょう - 001111
        assert_eq!(make_single_accent("じょ", 1, 0, 4), "00");
        assert_eq!(make_single_accent("う", 2, 0, 4), "1");
        assert_eq!(make_single_accent("きょ", 3, 0, 4), "11");
        assert_eq!(make_single_accent("う", 4, 0, 4), "1");
    }

    #[test]
    fn heiban_case1_new() {
        // がくせい - 0111
        assert_eq!(make_single_accent("が", 1, 0, 4), "0");
        assert_eq!(make_single_accent("く", 2, 0, 4), "1");
        assert_eq!(make_single_accent("せ", 3, 0, 4), "1");
        assert_eq!(make_single_accent("い", 4, 0, 4), "1");
    }

    #[test]
    fn heiban_case2_new() {
        // かいしゃ - 0111
        assert_eq!(make_single_accent("か", 1, 0, 4), "0");
        assert_eq!(make_single_accent("い", 2, 0, 4), "1");
        assert_eq!(make_single_accent("しゃ", 3, 0, 4), "11");
    }

    #[test]
    fn heiban_case3_new() {
        // みず - 01
        assert_eq!(make_single_accent("み", 1, 0, 2), "0");
        assert_eq!(make_single_accent("ず", 2, 0, 2), "1");
    }

    #[test]
    fn heiban_case4_new() {
        // しゅっせき - 00111
        assert_eq!(make_single_accent("しゅ", 1, 0, 4), "00");
        assert_eq!(make_single_accent("っ", 2, 0, 4), "1");
        assert_eq!(make_single_accent("せ", 3, 0, 4), "1");
        assert_eq!(make_single_accent("き", 4, 0, 4), "1");
    }

    // 中高 - low high then drop low before end of the word
    #[test]
    fn nakadaka_case5_old() {
        // たてもの - 0200
        assert_eq!(make_single_accent("た", 1, 2, 4), "0");
        assert_eq!(make_single_accent("て", 2, 2, 4), "2");
        assert_eq!(make_single_accent("も", 3, 2, 4), "0");
        assert_eq!(make_single_accent("の", 4, 2, 4), "0");
    }

    #[test]
    fn nakadaka_case6_old() {
        // たてもの - 0120
        assert_eq!(make_single_accent("た", 1, 3, 4), "0");
        assert_eq!(make_single_accent("て", 2, 3, 4), "1");
        assert_eq!(make_single_accent("も", 3, 3, 4), "2");
        assert_eq!(make_single_accent("の", 4, 3, 4), "0");
    }

    #[test]
    fn nakadaka_case1_new() {
        // おかし (お菓子) - 020
        assert_eq!(make_single_accent("お", 1, 2, 3), "0");
        assert_eq!(make_single_accent("か", 2, 2, 3), "2");
        assert_eq!(make_single_accent("し", 3, 2, 3), "0");
    }

    #[test]
    fn nakadaka_case2_new() {
        // にほんじん - 01120
        assert_eq!(make_single_accent("に", 1, 4, 5), "0");
        assert_eq!(make_single_accent("ほ", 2, 4, 5), "1");
        assert_eq!(make_single_accent("ん", 3, 4, 5), "1");
        assert_eq!(make_single_accent("じ", 4, 4, 5), "2");
        assert_eq!(make_single_accent("ん", 5, 4, 5), "0");
    }

    #[test]
    fn nakadaka_case3_new() {
        // しんぎょうそう - 0112000
        assert_eq!(make_single_accent("し", 1, 3, 6), "0");
        assert_eq!(make_single_accent("ん", 2, 3, 6), "1");
        assert_eq!(make_single_accent("ぎょ", 3, 3, 6), "12");
        assert_eq!(make_single_accent("う", 4, 3, 6), "0");
        assert_eq!(make_single_accent("そ", 5, 3, 6), "0");
        assert_eq!(make_single_accent("う", 6, 3, 6), "0");
    }

    #[test]
    fn nakadaka_case4_new() {
        // しゃっきん - 00120
        assert_eq!(make_single_accent("しゃ", 1, 3, 4), "00");
        assert_eq!(make_single_accent("っ", 2, 3, 4), "1");
        assert_eq!(make_single_accent("き", 3, 3, 4), "2");
        assert_eq!(make_single_accent("ん", 4, 3, 4), "0");
    }

    // 尾高 - low high then drop low at the end of the word
    #[test]
    fn odaka_case3_old() {
        // さむけ - 012
        assert_eq!(make_single_accent("さ", 1, 3, 3), "0");
        assert_eq!(make_single_accent("む", 2, 3, 3), "1");
        assert_eq!(make_single_accent("け", 3, 3, 3), "2");
    }

    #[test]
    fn odaka_case1_new() {
        // おとうと - 0112
        assert_eq!(make_single_accent("お", 1, 4, 4), "0");
        assert_eq!(make_single_accent("か", 2, 4, 4), "1");
        assert_eq!(make_single_accent("し", 3, 4, 4), "1");
        assert_eq!(make_single_accent("し", 4, 4, 4), "2");
    }

    #[test]
    fn odaka_case2_new() {
        // ことば - 012
        assert_eq!(make_single_accent("こ", 1, 3, 3), "0");
        assert_eq!(make_single_accent("と", 2, 3, 3), "1");
        assert_eq!(make_single_accent("ば", 3, 3, 3), "2");
    }

    #[test]
    fn odaka_case3_new() {
        // はな - 02
        assert_eq!(make_single_accent("こ", 1, 3, 3), "0");
        assert_eq!(make_single_accent("と", 2, 3, 3), "1");
        assert_eq!(make_single_accent("ば", 3, 3, 3), "2");
    }

    // end of nhk pitch generation tests
    //粉骨砕身␟紛骨砕身␟粉骨砕身␟粉骨砕心␞
    //ふんこつさいしん␞ふん'こ[Dev]つ[Akz]さい'しん␞0,0—0,1—0␞LHHHHHHHH,LHHHLHHHH,HLLLLHHHH
    #[test]
    fn gen_nhk_accent_simple_norm() {
        assert_eq!(generate_nhk_accent_from_pitch_number("やちん", 1), "200");
        assert_eq!(generate_nhk_accent_from_pitch_number("きょうみ", 1), "1200");
        assert_eq!(
            generate_nhk_accent_from_pitch_number("じょうきょう", 0),
            "001111"
        );
        assert_eq!(generate_nhk_accent_from_pitch_number("さむけ", 3), "012");
        assert_eq!(generate_nhk_accent_from_pitch_number("かんき", 1), "200");
        assert_eq!(
            generate_nhk_accent_from_pitch_number("しんぎょうそう", 3),
            "0112000"
        );
        //assert_eq!(generate_nhk_accent("しんぎょうそう", "1-1-1"), "2012020");
    }

    #[test]
    fn gen_nhk_accent_simple_double_small() {
        assert_eq!(
            generate_nhk_accent_from_pitch_number("しゃっきん", 3),
            "00120"
        );
    }

    #[test]
    fn gen_nhk_accent_string_norm_case1() {
        assert_eq!(
            generate_nhk_accent_from_pitch_number_string(
                "がでんいんすい",
                "0-0",
                "が'でん[Akz]いん'すい"
            ),
            HiraNhkPitch {
                hiragana: "がでんいんすい".to_string(),
                nhk_accent: "0110111".to_string()
            }
        );
    }

    #[test]
    fn gen_nhk_accent_string_norm_case2() {
        assert_eq!(
            generate_nhk_accent_from_pitch_number_string(
                "がでんいんすい",
                "1-0",
                "が'でん[Akz]いん'すい"
            ),
            HiraNhkPitch {
                hiragana: "がでんいんすい".to_string(),
                nhk_accent: "2000111".to_string(),
            }
        );
    }

    #[test]
    fn gen_nhk_accent_string_norm_case3() {
        assert_eq!(
            generate_nhk_accent_from_pitch_number_string("まずもって", "1-1", "まず[Akz]もって"),
            HiraNhkPitch {
                hiragana: "まずもって".to_string(),
                nhk_accent: "20200".to_string(),
            }
        );
    }

    // these normally raise IndexError from the below script (but it shouldn't)
    // https://github.com/IllDepence/anki_add_pitch/blob/master/wadoku_parse.py
    #[test]
    fn gen_nhk_accent_string_panic_case1() {
        assert_eq!(
            generate_nhk_accent_from_pitch_number_string("しりめつれつ", "1-0", ""),
            HiraNhkPitch {
                hiragana: "しりめつれつ".to_string(),
                nhk_accent: "20200".to_string(),
            }
        );
    }

    #[test]
    fn gen_nhk_accent_string_panic_case2() {
        // this is a special entry https://www.wadoku.de/entry/view/10051248
        // click the 0-1-2 and 0-3 to see the hiragana reading change
        assert_eq!(
            generate_nhk_accent_from_pitch_number_string(
                "きょうせいひほけんしゃ",
                "1-3",
                "&lt;きょう'せい&gt;[Akz]ひ[Akz]ほ'けん'しゃ"
            ),
            HiraNhkPitch {
                hiragana: "きょうせいひ".to_string(),
                nhk_accent: "001112".to_string(),
            }
        );
    }

    #[test]
    fn gen_nhk_accent_string_dev_case1() {
        assert_eq!(
            generate_nhk_accent_from_pitch_number_string(
                "しんしょうひつばつ",
                "0-0",
                "しん'しょう[Akz][Dev]ひつ'ばつ"
            ),
            HiraNhkPitch {
                hiragana: "しんしょうひつばつ".to_string(),
                nhk_accent: "011110111".to_string(),
            }
        );
    }
}
