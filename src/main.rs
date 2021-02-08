use colored::Colorize;
use csv::WriterBuilder;
use difference::{Changeset, Difference};
use quick_xml::de::from_str;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use xkcd_unreachable::xkcd_unreachable;

//use std::process::Comm
//use rusqlite::{params, Connection};

#[derive(Debug, Deserialize)]
struct Vocab {
    kanji: String,
    sentence: String,
}

struct DictKanaFuri {
    anki_export: String,
    kana: String,
    furigana: String,
}

fn process_dictionary_form(
    original: &str,
    yomi_original: &str,
    dictionary_form: &str,
    word_pos: &str,
) -> DictKanaFuri {
    let mut dict_kana = String::new();
    let mut dict_furigana = String::new();

    let changeset = Changeset::new(original, dictionary_form, "");
    let tmp_furigana_reading = get_furigana_reading(original, yomi_original, false);

    for (_i, x) in changeset.diffs.iter().enumerate() {
        if let Difference::Add(text) = x {
            // for na-adj they add だ to the dictionary form which we don't want
            if text == "だ" && word_pos == "形容詞" {
                return DictKanaFuri {
                    anki_export: original.to_string(),
                    kana: yomi_original.to_string(),
                    furigana: tmp_furigana_reading.to_string(),
                };
            } else {
                dict_kana += format!("{}{}", yomi_original, text).as_str();
                dict_furigana += format!("{}{}", tmp_furigana_reading, text).as_str();
            }
        }
    }

    DictKanaFuri {
        anki_export: dictionary_form.to_string(),
        kana: dict_kana,
        furigana: dict_furigana,
    }
}

fn get_furigana_reading(kanji: &str, yomi: &str, plain_text: bool) -> String {
    let mut text = String::new();

    let changeset = Changeset::new(kanji, yomi, "");
    for (i, _x) in changeset.diffs.iter().enumerate() {
        if let Difference::Rem(kanji) = &changeset.diffs[i] {
            if let Difference::Add(furigana) = &changeset.diffs[i + 1] {
                if plain_text {
                    // THIS DOES NOT WORK WITH ANKI
                    text += format!("{}[{}]", kanji, furigana).as_str();
                } else {
                    text += format!("<ruby><rb>{}<rt>{}</ruby>", kanji, furigana).as_str();
                }
            }
        } else if let Difference::Same(same) = &changeset.diffs[i] {
            text += same;
        }
    }
    text
}

#[derive(Debug, Deserialize, PartialEq)]
struct GdictRoot {
    headword: Vec<String>,
}

fn parse_gdict_xml_output(path: &str) {
    let s = fs::read_to_string(path).unwrap();

    let root: GdictRoot = from_str(&s).expect("something went wrong deser");
    let mut vocabs = String::new();
    let mut sentences = String::new();
    let mut sentence_count = -1;

    for (_i, x) in root.headword.iter().enumerate() {
        let v: Vec<char> = x.chars().collect();
        if v.len() > 7 {
            sentence_count += 1;
            sentences += format!("{}\n", x).as_str();
        } else {
            vocabs += format!("{:04} {}\n", sentence_count, x).as_str();
        }
    }
    let mut vocab_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("vocabs.txt")
        .unwrap();
    vocab_file.write_all(vocabs.as_bytes()).unwrap();

    let mut sentence_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("sentences.txt")
        .unwrap();
    sentence_file.write_all(sentences.as_bytes()).unwrap();
}

#[derive(Debug)]
struct AnkiExport {
    word: WordInformation,
    sentence: String,
    sentence_furigana: String,
}

#[derive(Debug)]
struct WordInformation {
    original: String,
    dictionary_form: String,
    reading_kana: String,     // for dictionary form, not original
    reading_furigana: String, // for dictionary form, not original
    pos: String,
    pos_information: String,
}

fn parse_jumanpp_output(
    output_path: &str,
    word_list: &str,
    debug_path: &str,
    result_counter_path: &str,
) -> Vec<AnkiExport> {
    let juman_output = fs::read_to_string(output_path).unwrap();
    let words_string = fs::read_to_string(word_list).unwrap();

    let re_ignore = Regex::new(r"^@").unwrap(); // ignore line starts with @
    let mut current_sentence_count = 0;
    let mut sentence = String::new();
    let mut sentence_furigana = String::new();
    let mut saved_words_information: Vec<WordInformation> = Vec::new();
    let mut dedupe_vec = Vec::new();

    let mut ret: Vec<AnkiExport> = Vec::new();
    let mut debug_text = String::new();
    let mut result_counter_text = String::new();

    for (_i, x) in juman_output.lines().enumerate() {
        if x == "EOS" {
            for word in saved_words_information {
                ret.push(AnkiExport {
                    word: word,
                    sentence: format!("{}", sentence).to_string(),
                    sentence_furigana: format!("{}", sentence_furigana).to_string(),
                });
            }
            current_sentence_count += 1;
            // reset
            sentence = "".to_string();
            sentence_furigana = "".to_string();
            saved_words_information = Vec::new();
        } else if !re_ignore.is_match(x) {
            let v: Vec<&str> = x.split(" ").collect();

            let dictionary_form_sanitized = &mut v[2].to_string();
            if v[3] == "形容詞" || v[3] == "助動詞" {
                // adjective or aux verb
                dictionary_form_sanitized.pop();
            }
            let tmp_re = Regex::new(
                format!(
                    "{} ({}|{})\n",
                    current_sentence_count, v[0], dictionary_form_sanitized
                )
                .as_str(),
            )
            .unwrap();

            debug_text += format!(
                "current sentence_count: {:04}, v[0]: {}, current regex: {}",
                current_sentence_count, v[0], tmp_re
            )
            .as_str();

            let mut highlight_text = String::new();

            if tmp_re.is_match(&words_string) && !dedupe_vec.contains(&v[0]) {
                debug_text += format!(
                    "GOT THROUGH => sentence_count: {:04}, v[0]: {}, current regex: {}",
                    current_sentence_count, v[0], tmp_re
                )
                .as_str();
                result_counter_text += format!("{:04} {}\n", current_sentence_count, v[2]).as_str();

                highlight_text = v[0].to_string();
                dedupe_vec.push(&v[0]);

                let p = process_dictionary_form(&v[0], &v[1], &v[2], &v[3]);
                saved_words_information.push(WordInformation {
                    original: v[0].to_string(),
                    dictionary_form: p.anki_export,
                    reading_kana: p.kana,
                    reading_furigana: p.furigana,
                    pos: v[3].to_string(),
                    pos_information: v[5].to_string(),
                });
            }

            let kanji_count = v[0]
                .chars()
                .filter(kanji::is_kanji)
                .collect::<Vec<char>>()
                .len();

            sentence += &v[0];

            if kanji_count > 0 {
                sentence_furigana += &get_furigana_reading(&v[0], &v[1], false);
            } else {
                sentence_furigana += &v[0];
            }
        }
    }

    let mut debug_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(debug_path)
        .unwrap();
    debug_file.write_all(debug_text.as_bytes()).unwrap();

    let mut result_counter_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(result_counter_path)
        .unwrap();
    result_counter_file
        .write_all(result_counter_text.as_bytes())
        .unwrap();

    ret
}

fn easy_counter_diff(file_path: &str) -> String {
    let s = fs::read_to_string(file_path).unwrap();
    let mut current_sentence_count = 0;
    let mut ret = String::new();

    let re = Regex::new(r"(\d+)").unwrap();
    let caps = re.captures_iter(&s);
    let mut count = 0;
    for cap in caps {
        if cap[1].parse::<i32>().unwrap() == current_sentence_count {
            count += 1;
        } else {
            ret += format!("{:04} {}\n", current_sentence_count, count).as_str();
            count = 0;
            current_sentence_count += 1;
        }
    }
    format!("{}{:04} {}", ret, current_sentence_count, count)
}

fn get_diff(original: &str, output: &str) -> String {
    let s1 = easy_counter_diff(original);
    let s2 = easy_counter_diff(output);

    let changeset = Changeset::new(&s1, &s2, "");
    let mut ret_text = String::new();
    for (_i, x) in changeset.diffs.iter().enumerate() {
        if let Difference::Same(text) = x {
            let mut tmp_vec: Vec<&str> = text.split("\n").collect();
            if let Some(sentence_count) = tmp_vec.pop() {
                ret_text += format!("{}", sentence_count).as_str();
            }
        } else if let Difference::Rem(text) = x {
            ret_text = format!("{}{}", ret_text, text.red());
        } else if let Difference::Add(text) = x {
            ret_text = format!("{}{}\n", ret_text, text.green());
        }
    }
    ret_text
}

#[derive(Debug, Serialize, Deserialize)]
struct MiningCard<'a> {
    vocab_kanji: &'a str,
    vocab_kanji_migaku: &'a str,
    vocab_furigana: &'a str,
    vocab_kana: &'a str,
    vocab_def_en: Option<&'a str>,
    vocab_def_ja: Option<&'a str>,
    vocab_audio: Option<&'a str>,
    vocab_pos: &'a str,
    vocab_pos_info: &'a str,
    pitch_accent: Option<u8>,
    picture: Option<&'a str>,
    sentence: &'a str,
    sentence_migaku: &'a str,
    sentence_furigana: &'a str,
    sentence_def: Option<&'a str>,
    sentence_audio: Option<&'a str>,
    hint: Option<&'a str>,
    extra_info: Option<&'a str>,
    kanjified: Option<&'a str>,
    freq_2016_ja: Option<u32>,
    freq_narou: Option<u32>,
    freq_anime_jdrama: Option<u32>,
    freq_netflix: Option<u32>,
}

fn main() {
    //let matches = App::new(PROGRAM_NAME)
    //    .setting(AppSettings::DisableHelpSubcommand)
    //    .version(crate_version!())
    //    .author(crate_authors!())
    //    .about(crate_description!())
    //    .arg(
    //        Arg::new("input")
    //            .about("the xml output file from goldendict")
    //            .required(true),
    //    )
    //    .get_matches();

    //if let Some(xml_file) = matches.value_of("input") {
    //    parse_gdict_xml_output(xml_file);
    //    Command::new("jumanpp")
    //        .arg("sentences.txt")
    //        .arg("-o")
    //        .arg("jumanpp.txt")
    //        .spawn()
    //        .expect("jumanpp command failed to start");
    //    let v: Vec<AnkiExport> = parse_jumanpp_output("jumanpp.txt", "vocabs.txt");
    //    let text = get_diff("vocabs.txt", "pad.txt");
    //} else {
    //    xkcd_unreachable!();
    //}
    let v: Vec<AnkiExport> =
        parse_jumanpp_output("jumanpp.txt", "vocabs.txt", "debug.txt", "result.txt");

    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_writer(vec![]);
    for i in v {
        let re_bold = Regex::new(format!("(?P<kanji>{})", &i.word.original).as_str()).unwrap();
        let re_bold_furigana =
            Regex::new(format!("(?P<kanji>{})", regex::escape(&i.word.reading_furigana)).as_str())
                .unwrap();
        let bold_sentence = re_bold.replace_all(&i.sentence, "<b>$kanji</b>");
        let bold_sentence_furigana =
            re_bold_furigana.replace_all(&i.sentence_furigana, "<b>$kanji</b>");
        wtr.serialize(MiningCard {
            vocab_kanji: &i.word.dictionary_form,
            vocab_kanji_migaku: &i.word.dictionary_form,
            vocab_furigana: &i.word.reading_furigana,
            vocab_kana: &i.word.reading_kana,
            vocab_def_en: None,
            vocab_def_ja: None,
            vocab_audio: None,
            vocab_pos: &i.word.pos,
            vocab_pos_info: &i.word.pos_information,
            pitch_accent: None,
            picture: None,
            sentence: &bold_sentence,
            sentence_migaku: &bold_sentence,
            sentence_furigana: &i.sentence_furigana,
            sentence_def: None,
            sentence_audio: None,
            hint: None,
            extra_info: None,
            kanjified: None,
            freq_2016_ja: None,
            freq_narou: None,
            freq_anime_jdrama: None,
            freq_netflix: None,
        });
    }
    let data = String::from_utf8(wtr.into_inner().expect("could not wrap into inner"))
        .expect("could not convert to utf8");
    println!("{}", data);
}
