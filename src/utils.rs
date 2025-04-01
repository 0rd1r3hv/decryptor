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

pub fn get_mac_addresses() -> Vec<Vec<u8>> {
    use crate::param::HEX_CAHRS;
    use std::ffi::c_void;
    use std::ptr::null_mut;

    #[repr(C)]
    struct IP_ADAPTER_INFO {
        next: *mut IP_ADAPTER_INFO,
        comboindex: u32,
        adapter_name: [u8; 260],
        description: [u8; 132],
        address_length: u32,
        address: [u8; 8],
        index: u32,
        _type: u32,
        dhcp_enabled: u32,
        current_ip_address: *mut c_void,
        ip_address_list: IP_ADDR_STRING,
        gateway_list: IP_ADDR_STRING,
        dhcp_server: IP_ADDR_STRING,
        have_wins: bool,
        primary_wins_server: IP_ADDR_STRING,
        secondary_wins_server: IP_ADDR_STRING,
        lease_obtained: i64,
        lease_expires: i64,
    }

    #[repr(C)]
    struct IP_ADDR_STRING {
        next: *mut IP_ADDR_STRING,
        ip_address: [u8; 16],
        ip_mask: [u8; 16],
        context: u32,
    }

    #[link(name = "iphlpapi")]
    unsafe extern "system" {
        fn GetAdaptersInfo(adapter_info: *mut IP_ADAPTER_INFO, size_pointer: *mut u32) -> u32;
    }

    let mut mac_addresses = Vec::new();
    unsafe {
        let mut size: u32 = 0;
        let result = GetAdaptersInfo(null_mut(), &mut size);
        if result == 111 && size > 0 {
            let mut buffer = vec![0u8; size as usize];
            let adapter_info = buffer.as_mut_ptr().cast::<IP_ADAPTER_INFO>();
            let result = GetAdaptersInfo(adapter_info, &mut size);
            if result == 0 {
                let mut current = adapter_info;
                while !current.is_null() {
                    let info = &*current;
                    if info.address_length > 0 {
                        let mac = info.address[..info.address_length as usize]
                            .iter()
                            .flat_map(|&byte| {
                                [
                                    HEX_CAHRS[(byte >> 4) as usize],
                                    HEX_CAHRS[(byte & 0x0F) as usize],
                                ]
                            })
                            .collect();
                        mac_addresses.push(mac);
                    }
                    current = info.next;
                }
            }
        }
    }
    mac_addresses
}

pub fn get_disk_info() -> Vec<u8> {
    use std::ffi::c_void;
    #[repr(C)]
    struct STORAGE_PROPERTY_QUERY {
        property_id: u32,
        query_type: u32,
        additional_parameters: [u8; 1],
    }

    #[repr(C)]
    struct STORAGE_DESCRIPTOR_HEADER {
        version: u32,
        size: u32,
    }

    #[repr(C)]
    struct STORAGE_DEVICE_DESCRIPTOR {
        version: u32,
        size: u32,
        device_type: u8,
        device_type_modifier: u8,
        removable_media: u8,
        command_queueing: u8,
        vendor_id_offset: u32,
        product_id_offset: u32,
        product_revision_offset: u32,
        serial_number_offset: u32,
        bus_type: u8,
        raw_properties_length: u32,
        raw_device_properties: [u8; 1],
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateFileW(
            file_name: *const u16,
            desired_access: u32,
            share_mode: u32,
            security_attributes: *const std::ffi::c_void,
            creation_disposition: u32,
            flags_and_attributes: u32,
            template_file: *const std::ffi::c_void,
        ) -> *mut std::ffi::c_void;

        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;

        fn DeviceIoControl(
            device: *mut std::ffi::c_void,
            io_control_code: u32,
            in_buffer: *mut std::ffi::c_void,
            in_buffer_size: u32,
            out_buffer: *mut std::ffi::c_void,
            out_buffer_size: u32,
            bytes_returned: *mut u32,
            overlapped: *mut std::ffi::c_void,
        ) -> i32;
    }

    const FILE_SHARE_READ: u32 = 0x00000001;
    const FILE_SHARE_WRITE: u32 = 0x00000002;
    const OPEN_EXISTING: u32 = 3;
    const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D1400;
    const STORAGE_PROPERTY_STANDARD: u32 = 0;
    const STORAGE_QUERY_PROPERTY_STANDARD: u32 = 0;

    let mut result = Vec::new();

    unsafe {
        let device_path = r"\\.\PhysicalDrive0"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        let handle = CreateFileW(
            device_path.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null(),
        );

        if !handle.is_null() {
            let mut query = STORAGE_PROPERTY_QUERY {
                property_id: STORAGE_PROPERTY_STANDARD,
                query_type: STORAGE_QUERY_PROPERTY_STANDARD,
                additional_parameters: [0],
            };

            let mut bytes_returned: u32 = 0;
            let mut header = STORAGE_DESCRIPTOR_HEADER {
                version: 0,
                size: 0,
            };

            let success = DeviceIoControl(
                handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                (&mut query as *mut STORAGE_PROPERTY_QUERY).cast::<c_void>(),
                std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                (&mut header as *mut STORAGE_DESCRIPTOR_HEADER).cast::<c_void>(),
                std::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32,
                &mut bytes_returned,
                std::ptr::null_mut(),
            );

            if success != 0 {
                let mut buffer = vec![0u8; header.size as usize];
                let success = DeviceIoControl(
                    handle,
                    IOCTL_STORAGE_QUERY_PROPERTY,
                    (&mut query as *mut STORAGE_PROPERTY_QUERY).cast::<c_void>(),
                    std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                    buffer.as_mut_ptr().cast::<c_void>(),
                    header.size,
                    &mut bytes_returned,
                    std::ptr::null_mut(),
                );

                if success != 0 {
                    let descriptor = buffer.as_ptr().cast::<STORAGE_DEVICE_DESCRIPTOR>();
                    let descriptor = &*descriptor;

                    if descriptor.serial_number_offset > 0 {
                        let offset = descriptor.serial_number_offset as usize;
                        let mut serial_number = Vec::new();
                        let mut i = offset;
                        while i < buffer.len() - 1 && buffer[i] != 0 && buffer[i + 1] != 0 {
                            serial_number.push(buffer[i + 1]);
                            serial_number.push(buffer[i]);
                            i += 2;
                        }
                        if buffer[i] != 0 {
                            serial_number.push(buffer[i]);
                        }
                        result.extend_from_slice(&serial_number);
                    }

                    if descriptor.product_id_offset > 0 {
                        let offset = descriptor.product_id_offset as usize;
                        let mut product_id = Vec::new();
                        let mut i = offset;
                        while i < buffer.len() && buffer[i] != 0 {
                            product_id.push(buffer[i]);
                            i += 1;
                        }
                        result.extend_from_slice(&product_id);
                    }

                    if descriptor.product_revision_offset > 0 {
                        let offset = descriptor.product_revision_offset as usize;
                        let mut revision = Vec::new();
                        let mut i = offset;
                        while i < buffer.len() && buffer[i] != 0 {
                            revision.push(buffer[i]);
                            i += 1;
                        }
                        result.extend_from_slice(&revision);
                    }
                }
            }
            CloseHandle(handle);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_mac_addresses() {
        let first_halves = get_mac_addresses();
        for first_half in first_halves {
            println!("{}", String::from_utf8_lossy(&first_half));
        }
    }

    #[test]
    fn test_get_disk_info() {
        let second_half = get_disk_info();
        println!("{}", String::from_utf8_lossy(&second_half));
    }
}
