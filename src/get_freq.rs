use rusqlite::{params, Connection};

pub fn get_freq_narou(file_path: &str, word: &str, reading: &str) -> Option<u32> {
    let conn = Connection::open(file_path).expect("could not open database file");
    let query_result: rusqlite::Result<u32> = conn.query_row_and_then(
        "SELECT freq FROM narou WHERE word=(?1) AND reading=(?2)",
        params![word, reading],
        |row| row.get(0),
    );
    match query_result {
        Ok(freq) => {
            return Some(freq);
        }
        Err(_e) => {
            let mut tmp_word = word.to_string();
            tmp_word.pop();
            let mut tmp_reading = reading.to_string();
            tmp_reading.pop();
            let query2_result: rusqlite::Result<u32> = conn.query_row_and_then(
                "SELECT freq FROM narou WHERE word=(?1) AND reading=(?2)",
                params![tmp_word, tmp_reading],
                |row| row.get(0),
            );
            match query2_result {
                Ok(freq) => {
                    return Some(freq);
                }
                Err(_e) => {
                    return None;
                }
            }
        }
    }
}

pub fn get_freq_anime_jdrama(file_path: &str, word: &str, table_name: &str) -> Option<u32> {
    let conn = Connection::open(file_path).expect("could not open database file");
    let query_result: rusqlite::Result<u32> = conn.query_row_and_then(
        format!("SELECT freq FROM {} WHERE word=(?1)", table_name).as_str(),
        params![word],
        |row| row.get(0),
    );
    match query_result {
        Ok(freq) => {
            return Some(freq);
        }
        Err(_e) => {
            return None;
        }
    }
}

pub fn get_freq_2016(file_path: &str, word: &str) -> Option<u32> {
    let conn = Connection::open(file_path).expect("could not open database file");
    let query_result: rusqlite::Result<u32> = conn.query_row_and_then(
        "SELECT freq FROM freq2016 WHERE word=(?1)",
        params![word],
        |row| row.get(0),
    );
    match query_result {
        Ok(freq) => {
            return Some(freq);
        }
        Err(_e) => {
            let mut tmp_word = word.to_string();
            tmp_word.pop();
            let query2_result: rusqlite::Result<u32> = conn.query_row_and_then(
                "SELECT freq FROM freq2016 WHERE word=(?1)",
                params![tmp_word],
                |row| row.get(0),
            );
            match query2_result {
                Ok(freq) => {
                    return Some(freq);
                }
                Err(_e) => {
                    return None;
                }
            }
        }
    }
}
