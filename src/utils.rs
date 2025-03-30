use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Seek, SeekFrom};

fn get_filenames_with_suffix(suffix: &str, path: &str) -> Vec<String> {
    let mut filenames = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == suffix {
                        if let Some(file_name) = path.file_name() {
                            if let Some(name) = file_name.to_str() {
                                filenames.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    filenames
}

fn get_name(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut name = String::new();

    file.seek(SeekFrom::End(-120))?;
    while let Ok(num) = file.read_u16::<LittleEndian>() {
        match num {
            0 => break,
            _ => name.push(num as u8 as char),
        }
    }
    Ok(name)
}

pub fn get_name_to_filename(path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let filenames = get_filenames_with_suffix("mflac", path);
    for filename in filenames {
        let name = get_name(&format!("{}\\{}", path, filename)).unwrap();
        map.insert(name, filename);
    }
    map
}

pub fn parse_next_integer(buf: &[u8]) -> Option<(usize, usize)> {
    match buf.len() {
        0 => None,
        _ => Some(
            buf.iter()
                .enumerate()
                .scan(true, |state, (i, &num)| match *state {
                    true => {
                        *state = num >> 7 == 1;
                        Some((i, num))
                    }
                    _ => None,
                })
                .fold((0, 0), |acc: (usize, usize), (i, num)| {
                    (acc.0 | ((num & 0x7F) as usize) << (i * 7), acc.1 + 1)
                }),
        ),
    }
}

pub fn parse_next_kv(buf: &[u8]) -> Option<(&[u8], &[u8], &[u8])> {
    let (key_len, key_len_len) = parse_next_integer(buf)?;
    let mut total_len = key_len_len;
    let key = &buf[total_len..total_len + key_len];
    total_len += key_len;
    let (_, junk_len) = parse_next_integer(&buf[total_len..])?;
    total_len += junk_len;
    let (value_len, value_len_len) = parse_next_integer(&buf[total_len..])?;
    total_len += value_len_len;
    let value = &buf[total_len..total_len + value_len];
    total_len += value_len;
    Some((&buf[total_len..], key, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_name() {
        let prefix = ".";
        let files = get_filenames_with_suffix("mflac", prefix);

        for file in files {
            let name = get_name(&format!("{}\\{}", prefix, file)).unwrap();
            println!("{}", name);
        }
    }
}
