use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::fs::{File, OpenOptions};
use std::io::{Read, BufReader, BufWriter, Seek, SeekFrom, Write};

#[derive(Debug)]
pub struct Entry {
    pub salt: [u8; 16],
    pub kek_nonce: [u8; 12],
    pub encrypted_cek: [u8; 32],
    pub encrypted_cek_nonce: [u8; 12],
}

#[derive(Debug)]
pub struct FileHeader {
    pub magic: [u8; 4],
    pub password_count: u16,
    pub entries: Vec<Entry>,
    pub encrypted_verify_block: Vec<u8>, // 128B
    pub verify_block_hmac: [u8; 32],
    pub file_hmac: [u8; 32],
    pub metadata_len: u32,
    pub metadata: Vec<u8>,
}

pub fn parse_header(path: &str) -> Result<FileHeader> {
    let mut file = BufReader::new(File::open(path).context("打开文件失败")?);

    // 1. Magic (4B)
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;

    // 2. Password count (2B, BE)
    let password_count = file.read_u16::<BigEndian>()?;

    // 3. Entries
    let mut entries = Vec::with_capacity(password_count as usize);
    for _ in 0..password_count {
        let mut salt = [0u8; 16];
        let mut kek_nonce = [0u8; 12];
        let mut encrypted_cek = [0u8; 32];
        let mut encrypted_cek_nonce = [0u8; 12];

        file.read_exact(&mut salt)?;
        file.read_exact(&mut kek_nonce)?;
        file.read_exact(&mut encrypted_cek)?;
        file.read_exact(&mut encrypted_cek_nonce)?;

        entries.push(Entry {
            salt,
            kek_nonce,
            encrypted_cek,
            encrypted_cek_nonce,
        });
    }

    // 4. Encrypted Verify Block (128B)
    let mut encrypted_verify_block = vec![0u8; 128];
    file.read_exact(&mut encrypted_verify_block)?;

    // 5. Verify Block HMAC (32B)
    let mut verify_block_hmac = [0u8; 32];
    file.read_exact(&mut verify_block_hmac)?;

    // 6. File HMAC (32B)
    let mut file_hmac = [0u8; 32];
    file.read_exact(&mut file_hmac)?;

    // 7. Metadata Length (4B, BE) and Metadata
    let mut metadata_len_buf = [0u8; 4];
    let metadata_len = match file.read_exact(&mut metadata_len_buf) {
        Ok(()) => u32::from_be_bytes(metadata_len_buf),
        Err(_) => 0, // 向后兼容：旧格式没有元数据
    };
    let mut metadata = vec![0u8; metadata_len as usize];
    if metadata_len > 0 {
        file.read_exact(&mut metadata).context("读取元数据失败")?;
    }

    Ok(FileHeader {
        magic,
        password_count,
        entries,
        encrypted_verify_block,
        verify_block_hmac,
        file_hmac,
        metadata_len,
        metadata,
    })
}

pub fn print_header_info(path: &str) -> Result<()> {
    let mut file = BufReader::new(File::open(path).context("打开文件失败")?);

    // 1. Magic (4B)
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    println!("Magic: {:?}", String::from_utf8_lossy(&magic));

    // 2. Password count (2B, BE)
    let pw_count = file.read_u16::<BigEndian>()?;
    println!("Password Count: {}", pw_count);

    // 3. Entries
    for i in 0..pw_count {
        let mut salt = [0u8; 16];
        let mut kek_nonce = [0u8; 12];
        let mut enc_cek = [0u8; 32];
        let mut enc_cek_nonce = [0u8; 12];

        file.read_exact(&mut salt)?;
        file.read_exact(&mut kek_nonce)?;
        file.read_exact(&mut enc_cek)?;
        file.read_exact(&mut enc_cek_nonce)?;

        println!("--- Entry {} ---", i);
        println!("Salt           : {:02X?}", salt);
        println!("KEK Nonce      : {:02X?}", kek_nonce);
        println!("Encrypted CEK  : {:02X?}", enc_cek);
        println!("Enc CEK Nonce  : {:02X?}", enc_cek_nonce);
    }

    // 4. Encrypted Verify Block (128B)
    let mut encrypted_verify_block = vec![0u8; 128];
    file.read_exact(&mut encrypted_verify_block)?;
    println!("Encrypted Verify Block (128B): {:02X?}", &encrypted_verify_block[..16]); // 只打印前16字节

    // 5. Verify Block HMAC (32B)
    let mut verify_block_hmac = [0u8; 32];
    file.read_exact(&mut verify_block_hmac)?;
    println!("Verify Block HMAC: {:02X?}", verify_block_hmac);

    // 6. File HMAC (32B)
    let mut file_hmac = [0u8; 32];
    file.read_exact(&mut file_hmac)?;
    println!("File HMAC: {:02X?}", file_hmac);

    // 7. Metadata Length (4B, BE) and Metadata
    let mut metadata_len_buf = [0u8; 4];
    let metadata_len = match file.read_exact(&mut metadata_len_buf) {
        Ok(()) => u32::from_be_bytes(metadata_len_buf),
        Err(_) => 0, // 向后兼容：旧格式没有元数据
    };
    println!("Metadata Length: {}", metadata_len);
    if metadata_len > 0 {
        let mut metadata = vec![0u8; metadata_len as usize];
        file.read_exact(&mut metadata).context("读取元数据失败")?;
        // 尝试将元数据作为 UTF-8 打印，若失败则打印十六进制
        match String::from_utf8(metadata.clone()) {
            Ok(s) => println!("Metadata (UTF-8): {}", s),
            Err(_) => println!("Metadata (Hex): {:02X?}", metadata),
        }
    } else {
        println!("Metadata: None");
    }

    println!("(剩余部分为加密后的文件数据)");

    Ok(())
}

pub fn update_metadata(path: &str, new_metadata: Vec<u8>) -> Result<()> {
    // 1. 解析原始文件头
    let header = parse_header(path).context("解析文件头失败")?;

    // 2. 读取文件剩余数据（加密文件内容）
    let mut file = BufReader::new(File::open(path).context("打开文件失败")?);
    let header_len = 4 + 2 + (header.password_count as usize * (16 + 12 + 32 + 12)) + 128 + 32 + 32 + 4 + header.metadata_len as usize;
    file.seek(SeekFrom::Start(header_len as u64))?;
    let mut remaining_data = Vec::new();
    file.read_to_end(&mut remaining_data).context("读取剩余文件数据失败")?;

    // 3. 打开文件以写入，覆盖原文件
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .context("打开文件以写入失败")?,
    );

    // 4. 写入更新后的文件头
    // Magic (4B)
    file.write_all(&header.magic)?;
    // Password Count (2B, BE)
    file.write_u16::<BigEndian>(header.password_count)?;
    // Entries
    for entry in &header.entries {
        file.write_all(&entry.salt)?;
        file.write_all(&entry.kek_nonce)?;
        file.write_all(&entry.encrypted_cek)?;
        file.write_all(&entry.encrypted_cek_nonce)?;
    }
    // Encrypted Verify Block (128B)
    file.write_all(&header.encrypted_verify_block)?;
    // Verify Block HMAC (32B)
    file.write_all(&header.verify_block_hmac)?;
    // File HMAC (32B)
    file.write_all(&header.file_hmac)?;
    // Metadata Length (4B, BE)
    file.write_u32::<BigEndian>(new_metadata.len() as u32)?;
    // Metadata
    file.write_all(&new_metadata)?;

    // 5. 写入剩余的加密文件数据
    file.write_all(&remaining_data)?;
    file.flush().context("写入文件失败")?;

    Ok(())
}