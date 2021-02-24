use csv::{ReaderBuilder, WriterBuilder};
use regex::Regex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;

fn make_narou_db(file_path: &str, db_file_output: &str) {
    let s = fs::read_to_string(file_path).unwrap();
    let mut conn = Connection::open(db_file_output).expect("could not open database file");
    let tx = conn.transaction().expect("could not open transaction");
    //schema
    tx.execute(
        "CREATE TABLE narou (
            id integer primary key,
            word text not null,
            reading text not null,
            freq integer not null
            )",
        rusqlite::NO_PARAMS,
    )
    .expect("could not import schema");

    let re =
        Regex::new(r#"\["(.*?)",\s?"freq",\s?\{"reading":\s?"(.*?)",\s?"frequency":\s(\d+)\}\]"#)
            .unwrap();
    for cap in re.captures_iter(&s) {
        tx.execute(
            "INSERT INTO narou (word, reading, freq) VALUES (?1, ?2, ?3)",
            params![&cap[1], &cap[2], &cap[3]],
        )
        .expect("could not insert query");
    }

    tx.execute(
        "CREATE INDEX ix_word_narou ON narou (word COLLATE NOCASE)",
        rusqlite::NO_PARAMS,
    )
    .expect("could not create index");
    tx.commit().unwrap();
}

// this ONLY works for anime&jdrama and netflix json
// aka db_name should only takes netflix and anime_jdrama
fn make_freq_db(file_path: &str, db_file_output: &str, db_name: &str) {
    let s = fs::read_to_string(file_path).unwrap();
    let mut conn = Connection::open(db_file_output).expect("could not open database file");
    let tx = conn.transaction().expect("could not open transaction");
    //schema
    tx.execute(
        format!(
            "CREATE TABLE {} (
            id integer primary key,
            word text not null,
            freq integer not null
            )",
            db_name
        )
        .as_str(),
        rusqlite::NO_PARAMS,
    )
    .expect("could not import schema");

    let re = Regex::new(r#"\["(.*?)",\s?"freq",\s?(\d+)\]"#).unwrap();
    for cap in re.captures_iter(&s) {
        tx.execute(
            format!("INSERT INTO {} (word, freq) VALUES (?1, ?2)", db_name).as_str(),
            params![&cap[1], &cap[2]],
        )
        .expect("could not insert query");
    }

    tx.execute(
        format!(
            "CREATE INDEX ix_word_{} ON {} (word COLLATE NOCASE)",
            db_name, db_name
        )
        .as_str(),
        rusqlite::NO_PARAMS,
    )
    .expect("could not create index");
    tx.commit().unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
struct Freq2016 {
    word: String,
    data: String,
}

fn make_freq_2016_ja(file_path: &str, db_file_output: &str) {
    let s = fs::read_to_string(file_path).unwrap();
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_reader(s.as_bytes());
    let mut iter = rdr.deserialize();
    let re = Regex::new(r#"<div class="ce_js">(\d+)"#).unwrap();
    let re_occ = Regex::new(r#"<BR>(\d+) of 13,280,660"#).unwrap();

    let mut conn = Connection::open(db_file_output).expect("could not open database file");
    let tx = conn.transaction().expect("could not open transaction");
    //schema
    tx.execute(
        "CREATE TABLE freq2016 (
            id integer primary key,
            word text not null,
            freq integer not null,
            occ integer not null
            )",
        rusqlite::NO_PARAMS,
    )
    .expect("could not import schema");

    for result in iter {
        let record: Freq2016 = result.expect("could not get result");
        let cap = re.captures(&record.data).unwrap();
        let cap_occ = re_occ.captures(&record.data).unwrap();

        let freq_cap = cap.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let occ_cap = cap_occ.get(1).unwrap().as_str().parse::<u32>().unwrap();

        tx.execute(
            "INSERT INTO freq2016 (word, freq, occ) VALUES (?1, ?2, ?3)",
            params![&record.word, freq_cap, occ_cap],
        )
        .expect("could not insert query");
    }

    tx.execute(
        "CREATE INDEX ix_word_ja2016 ON freq2016 (word COLLATE NOCASE)",
        rusqlite::NO_PARAMS,
    )
    .expect("could not create index");
    tx.commit().unwrap();
}

#[derive(Debug, Deserialize)]
struct Wadoku {
    date: String,
    entry: Vec<WadokuEntry>,
}

#[derive(Debug, Deserialize)]
struct WadokuEntry {
    id: u32,
    form: WadokuEntryForm,
}

#[derive(Debug, Deserialize)]
struct WadokuEntryForm {
    #[serde(rename = "orth")]
    orths: Vec<WadokuOrth>,
    reading: WadokuEntryReading,
}

#[derive(Debug, Deserialize)]
struct WadokuOrth {
    midashigo: Option<String>,
    #[serde(rename = "$value")]
    orth_value: String,
}

#[derive(Debug, Deserialize)]
struct WadokuEntryReading {
    #[serde(rename = "hira")]
    hiragana_reading: String,
    #[serde(rename = "accent")]
    accents: Option<Vec<u8>>,
}

pub fn parse_wadoku_xml(file_path: &str) {
    let wadoku_xml = fs::read_to_string(file_path).unwrap();
    let wadoku: Wadoku = quick_xml::de::from_str(&wadoku_xml).expect("could not parse xml");
    println!("{:#?}", wadoku);
}
