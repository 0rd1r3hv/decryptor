use anyhow::{Context, Result};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{File, read_dir};
use std::path::{Path, PathBuf};

pub fn get_child_path_by_prfx_and_sfx(dir: &Path, prfx: &str, ext: &str) -> Result<Vec<PathBuf>> {
    Ok(read_dir(dir)
        .with_context(|| "Fail to read dir.")?
        .filter_map(|dir_ent| {
            let path = dir_ent.ok()?.path();
            path.file_name()
                .is_some_and(|name| {
                    name.to_str()
                        .is_some_and(|s| s.starts_with(prfx) && s.ends_with(ext))
                })
                .then_some(path)
        })
        .collect())
}

pub fn get_name_path_map(dir: &Path) -> Result<HashMap<String, PathBuf>> {
    let get_child_path_by_ext = |dir: &Path, ext: &OsStr| -> Result<Vec<PathBuf>> {
        Ok(read_dir(dir)
            .with_context(|| "Fail to read dir.")?
            .filter_map(|dir_ent| {
                let path = dir_ent.ok()?.path();
                path.extension().is_some_and(|e| e == ext).then_some(path)
            })
            .collect())
    };
    let get_embed_ent_name = |path: &Path| -> Result<String> {
        use std::io::{Seek, SeekFrom};
        let mut file = File::open(path).with_context(|| "Fail to open file.")?;
        file.seek(SeekFrom::End(-120))
            .with_context(|| "Fail to seek in file.")?;

        Ok({
            use byteorder::{LittleEndian, ReadBytesExt};
            core::iter::from_fn(|| file.read_u16::<LittleEndian>().ok())
                .take_while(|&num| num != 0)
                .map(|num| (num as u8) as char)
                .collect()
        })
    };

    get_child_path_by_ext(dir, "mflac".as_ref())?
        .into_iter()
        .map(|path| Ok((get_embed_ent_name(&path)?, path)))
        .collect::<anyhow::Result<HashMap<_, _>>>()
}

pub fn parse_next_kv(buf: &[u8]) -> Option<(&[u8], &[u8], &[u8])> {
    let parse_next_int = |buf: &[u8]| {
        core::ops::Not::not(buf.is_empty()).then_some(
            buf.iter()
                .enumerate()
                .scan(true, |state, (i, &num)| {
                    state.then_some({
                        *state = num >> 7 == 1;
                        (i, num)
                    })
                })
                .fold((0, 0), |acc, (i, num)| {
                    (acc.0 | (((num & 0x7F) as usize) << (i * 7)), acc.1 + 1)
                }),
        )
    };

    let (key_len, key_len_len) = parse_next_int(buf)?;
    let mut ofst = key_len_len;
    let key = &buf[ofst..ofst + key_len];
    ofst += key_len;
    ofst += parse_next_int(&buf[ofst..])?.1;
    let (val_len, val_len_len) = parse_next_int(&buf[ofst..])?;
    ofst += val_len_len;
    let val = &buf[ofst..ofst + val_len];
    ofst += val_len;
    Some((&buf[ofst..], key, val))
}
