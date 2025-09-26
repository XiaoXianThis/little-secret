use anyhow::{bail, Context, Result};
use argon2::{Argon2, Params, PasswordHasher};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chacha20::{ChaCha20, Key, Nonce};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use std::fs::{rename, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use cipher::{KeyIvInit, StreamCipher};
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

const HEADER_MAGIC: &[u8; 4] = b"WCRY";
const VERIFY_BLOCK_SIZE: usize = 128;
const ENTRY_SALT_LEN: usize = 16;
const ENTRY_KEK_NONCE_LEN: usize = 12;
const CEK_LEN: usize = 32;
const CEK_NONCE_LEN: usize = 12;
const ENTRY_ENCRYPTED_CEK_LEN: usize = CEK_LEN;
const ENTRY_ENCRYPTED_CEK_NONCE_LEN: usize = CEK_NONCE_LEN;
const ENTRY_SIZE: usize =
    ENTRY_SALT_LEN + ENTRY_KEK_NONCE_LEN + ENTRY_ENCRYPTED_CEK_LEN + ENTRY_ENCRYPTED_CEK_NONCE_LEN; // 72

fn derive_kek(password: &[u8], salt: &[u8]) -> Result<[u8; CEK_LEN]> {
    let params = Params::new(32 * 1024, 2, 4, Some(CEK_LEN as u32 as usize)).unwrap();
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut out = [0u8; CEK_LEN];
    argon2.hash_password_into(password, salt, &mut out).unwrap();
    Ok(out)
}

fn gen_nonce() -> [u8; CEK_NONCE_LEN] {
    let mut n = [0u8; CEK_NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut n);
    n
}

fn gen_cek() -> ([u8; CEK_LEN], [u8; CEK_NONCE_LEN]) {
    let mut cek = [0u8; CEK_LEN];
    rand::thread_rng().fill_bytes(&mut cek);
    (cek, gen_nonce())
}

fn chacha_xor(key: &[u8], nonce: &[u8], content: &[u8]) -> Result<Vec<u8>> {
    let key_slice = Key::from_slice(key);
    let nonce_slice = Nonce::from_slice(nonce);
    let mut cipher = ChaCha20::new(key_slice, nonce_slice);
    let mut out = content.to_vec();
    cipher.apply_keystream(&mut out);
    Ok(out)
}

pub fn encrypt_file(input_path: &str, output_path: &str, passwords: &[String], metadata: Option<&[u8]>) -> Result<()> {
    if passwords.is_empty() {
        bail!("至少需要一个密码");
    }
    if passwords.len() > 65535 {
        bail!("密码数量过多，最多支持 65535 个");
    }

    let mut input_file = File::open(input_path).context("打开输入文件失败")?;
    let mut output_file = File::create(output_path).context("创建输出文件失败")?;

    let (cek, cek_nonce) = gen_cek();
    let mut verify_block = vec![0u8; VERIFY_BLOCK_SIZE];
    rand::thread_rng().fill_bytes(&mut verify_block);
    let encrypted_verify_block = chacha_xor(&cek, &cek_nonce, &verify_block)?;
    let mut hmac_verify = HmacSha256::new_from_slice(&cek).expect("HMAC 初始化失败");
    hmac_verify.update(&verify_block);
    let verify_block_hmac = hmac_verify.finalize().into_bytes();

    let mut file_hmac_hasher = HmacSha256::new_from_slice(&cek).expect("HMAC 初始化失败");
    {
        let mut rdr = BufReader::new(&mut input_file);
        let mut buf = [0u8; 8192];
        loop {
            let n = rdr.read(&mut buf)?;
            if n == 0 {
                break;
            }
            file_hmac_hasher.update(&buf[..n]);
        }
    }
    let file_hmac = file_hmac_hasher.finalize().into_bytes();
    input_file.seek(SeekFrom::Start(0)).context("重置输入文件指针失败")?;

    let metadata_len = metadata.map_or(0, |m| m.len());
    let mut header = Vec::with_capacity(6 + passwords.len() * ENTRY_SIZE + VERIFY_BLOCK_SIZE + 32 + 32 + 4 + metadata_len);
    header.extend_from_slice(HEADER_MAGIC);
    header.write_u16::<BigEndian>(passwords.len() as u16)?;

    for pw in passwords {
        let mut salt = [0u8; ENTRY_SALT_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        let kek_nonce = gen_nonce();
        let kek = derive_kek(pw.as_bytes(), &salt)?;
        let encrypted_cek = chacha_xor(&kek, &kek_nonce, &cek)?;
        let encrypted_cek_nonce = chacha_xor(&kek, &kek_nonce, &cek_nonce)?;
        header.extend_from_slice(&salt);
        header.extend_from_slice(&kek_nonce);
        header.extend_from_slice(&encrypted_cek);
        header.extend_from_slice(&encrypted_cek_nonce);
    }

    header.extend_from_slice(&encrypted_verify_block);
    header.extend_from_slice(&verify_block_hmac);
    header.extend_from_slice(&file_hmac);
    header.write_u32::<BigEndian>(metadata_len as u32)?;
    if let Some(metadata) = metadata {
        header.extend_from_slice(metadata);
    }

    output_file.write_all(&header).context("写入头部失败")?;
    let mut cipher = ChaCha20::new(&cek.into(), &cek_nonce.into());
    let mut reader = BufReader::new(input_file);
    let mut writer = BufWriter::new(output_file);
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let mut chunk = buf[..n].to_vec();
        cipher.apply_keystream(&mut chunk);
        writer.write_all(&chunk)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(bool, Vec<u8>)> {
    let mut input_file = File::open(input_path).context("打开输入文件失败")?;
    let mut fixed_header = [0u8; 6];
    input_file.read_exact(&mut fixed_header).context("读取固定头部失败")?;
    if &fixed_header[..4] != HEADER_MAGIC {
        bail!("无效的文件头部，非 WCRY 格式");
    }
    let num_passwords = (&fixed_header[4..6]).read_u16::<BigEndian>()? as usize;
    let mut entries = vec![0u8; num_passwords * ENTRY_SIZE];
    input_file.read_exact(&mut entries).context("读取密码条目失败")?;
    let mut vb_and_hmac = vec![0u8; VERIFY_BLOCK_SIZE + 32 + 32];
    input_file.read_exact(&mut vb_and_hmac).context("读取验证块和 HMAC 失败")?;
    let encrypted_verify_block = &vb_and_hmac[..VERIFY_BLOCK_SIZE];
    let verify_block_hmac = &vb_and_hmac[VERIFY_BLOCK_SIZE..VERIFY_BLOCK_SIZE + 32];
    let original_file_hmac = &vb_and_hmac[VERIFY_BLOCK_SIZE + 32..];

    let mut metadata_len_buf = [0u8; 4];
    let metadata_len = match input_file.read_exact(&mut metadata_len_buf) {
        Ok(()) => u32::from_be_bytes(metadata_len_buf) as usize,
        Err(_) => 0, // 向后兼容：旧格式没有元数据
    };
    let mut metadata = vec![0u8; metadata_len];
    if metadata_len > 0 {
        input_file.read_exact(&mut metadata).context("读取元数据失败")?;
    }

    let mut found = false;
    let mut cek = [0u8; CEK_LEN];
    let mut cek_nonce = [0u8; CEK_NONCE_LEN];
    let pw_bytes = password.as_bytes();

    for i in 0..num_passwords {
        let off = i * ENTRY_SIZE;
        let salt = &entries[off..off + ENTRY_SALT_LEN];
        let kek_nonce = &entries[off + ENTRY_SALT_LEN..off + ENTRY_SALT_LEN + ENTRY_KEK_NONCE_LEN];
        let encrypted_cek = &entries[off + ENTRY_SALT_LEN + ENTRY_KEK_NONCE_LEN
            ..off + ENTRY_SALT_LEN + ENTRY_KEK_NONCE_LEN + ENTRY_ENCRYPTED_CEK_LEN];
        let encrypted_cek_nonce = &entries[off + ENTRY_SALT_LEN + ENTRY_KEK_NONCE_LEN + ENTRY_ENCRYPTED_CEK_LEN
            ..off + ENTRY_SIZE];

        let kek = match derive_kek(pw_bytes, salt) {
            Ok(k) => k,
            Err(_) => continue,
        };
        let cek_candidate = match chacha_xor(&kek, kek_nonce, encrypted_cek) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let cek_nonce_candidate = match chacha_xor(&kek, kek_nonce, encrypted_cek_nonce) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if cek_candidate.len() != CEK_LEN || cek_nonce_candidate.len() != CEK_NONCE_LEN {
            continue;
        }
        let verify_block_candidate = match chacha_xor(&cek_candidate, &cek_nonce_candidate, encrypted_verify_block) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let mut h = HmacSha256::new_from_slice(&cek_candidate).expect("HMAC 初始化失败");
        h.update(&verify_block_candidate);
        let sum = h.finalize().into_bytes();
        if sum.ct_eq(verify_block_hmac).unwrap_u8() == 1 {
            cek.copy_from_slice(&cek_candidate[..CEK_LEN]);
            cek_nonce.copy_from_slice(&cek_nonce_candidate[..CEK_NONCE_LEN]);
            found = true;
            break;
        }
    }

    if !found {
        bail!("提供的密码不匹配任何加密密钥");
    }

    let mut output_file = File::create(output_path).context("创建输出文件失败")?;
    let mut cipher = ChaCha20::new(&cek.into(), &cek_nonce.into());
    let mut hmac_hasher = HmacSha256::new_from_slice(&cek).expect("HMAC 初始化失败");
    let mut buf = [0u8; 8192];
    loop {
        let n = input_file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let mut chunk = buf[..n].to_vec();
        cipher.apply_keystream(&mut chunk);
        output_file.write_all(&chunk)?;
        hmac_hasher.update(&chunk);
    }
    let computed_hmac = hmac_hasher.finalize().into_bytes();
    let matched = computed_hmac.ct_eq(original_file_hmac).unwrap_u8() == 1;
    Ok((matched, metadata))
}

pub fn reset_passwords(encrypted_path: &str, old_password: &str, new_passwords: &[String]) -> Result<()> {
    if new_passwords.is_empty() {
        bail!("至少需要一个新密码");
    }
    if new_passwords.len() > 65535 {
        bail!("新密码数量过多，最多支持 65535 个");
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(encrypted_path)
        .context("打开加密文件失败")?;
    let mut fixed_header = [0u8; 6];
    file.read_exact(&mut fixed_header).context("读取固定头部失败")?;
    if &fixed_header[..4] != HEADER_MAGIC {
        bail!("无效的文件头部，非 WCRY 格式");
    }
    let old_num_passwords = (&fixed_header[4..6]).read_u16::<BigEndian>()? as usize;
    let mut old_entries = vec![0u8; old_num_passwords * ENTRY_SIZE];
    file.read_exact(&mut old_entries).context("读取旧密码条目失败")?;
    let mut vb_and_hmac = vec![0u8; VERIFY_BLOCK_SIZE + 32 + 32];
    file.read_exact(&mut vb_and_hmac).context("读取验证块和 HMAC 失败")?;
    let encrypted_verify_block = &vb_and_hmac[..VERIFY_BLOCK_SIZE];
    let verify_block_hmac = &vb_and_hmac[VERIFY_BLOCK_SIZE..VERIFY_BLOCK_SIZE + 32];
    let original_file_hmac = &vb_and_hmac[VERIFY_BLOCK_SIZE + 32..];

    let mut metadata_len_buf = [0u8; 4];
    let metadata_len = match file.read_exact(&mut metadata_len_buf) {
        Ok(()) => u32::from_be_bytes(metadata_len_buf) as usize,
        Err(_) => 0, // 向后兼容：旧格式没有元数据
    };
    let mut metadata = vec![0u8; metadata_len];
    if metadata_len > 0 {
        file.read_exact(&mut metadata).context("读取元数据失败")?;
    }

    let mut found = false;
    let mut cek = [0u8; CEK_LEN];
    let mut cek_nonce = [0u8; CEK_NONCE_LEN];
    let pw_bytes = old_password.as_bytes();

    for i in 0..old_num_passwords {
        let off = i * ENTRY_SIZE;
        let salt = &old_entries[off..off + ENTRY_SALT_LEN];
        let kek_nonce = &old_entries[off + ENTRY_SALT_LEN..off + ENTRY_SALT_LEN + ENTRY_KEK_NONCE_LEN];
        let encrypted_cek = &old_entries[off + ENTRY_SALT_LEN + ENTRY_KEK_NONCE_LEN
            ..off + ENTRY_SALT_LEN + ENTRY_KEK_NONCE_LEN + ENTRY_ENCRYPTED_CEK_LEN];
        let encrypted_cek_nonce = &old_entries[off + ENTRY_SALT_LEN + ENTRY_KEK_NONCE_LEN + ENTRY_ENCRYPTED_CEK_LEN
            ..off + ENTRY_SIZE];

        let kek = match derive_kek(pw_bytes, salt) {
            Ok(k) => k,
            Err(_) => continue,
        };
        let cek_candidate = match chacha_xor(&kek, kek_nonce, encrypted_cek) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let cek_nonce_candidate = match chacha_xor(&kek, kek_nonce, encrypted_cek_nonce) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if cek_candidate.len() != CEK_LEN || cek_nonce_candidate.len() != CEK_NONCE_LEN {
            continue;
        }
        let verify_block_candidate = match chacha_xor(&cek_candidate, &cek_nonce_candidate, encrypted_verify_block) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let mut h = HmacSha256::new_from_slice(&cek_candidate).expect("HMAC 初始化失败");
        h.update(&verify_block_candidate);
        let sum = h.finalize().into_bytes();
        if sum.ct_eq(verify_block_hmac).unwrap_u8() == 1 {
            cek.copy_from_slice(&cek_candidate[..CEK_LEN]);
            cek_nonce.copy_from_slice(&cek_nonce_candidate[..CEK_NONCE_LEN]);
            found = true;
            break;
        }
    }

    if !found {
        bail!("提供的旧密码不正确");
    }

    let mut new_header = Vec::with_capacity(6 + new_passwords.len() * ENTRY_SIZE + VERIFY_BLOCK_SIZE + 32 + 32 + 4 + metadata_len);
    new_header.extend_from_slice(HEADER_MAGIC);
    new_header.write_u16::<BigEndian>(new_passwords.len() as u16)?;
    for npw in new_passwords {
        let mut salt = [0u8; ENTRY_SALT_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        let new_kek_nonce = gen_nonce();
        let new_kek = derive_kek(npw.as_bytes(), &salt)?;
        let new_encrypted_cek = chacha_xor(&new_kek, &new_kek_nonce, &cek)?;
        let new_encrypted_cek_nonce = chacha_xor(&new_kek, &new_kek_nonce, &cek_nonce)?;
        new_header.extend_from_slice(&salt);
        new_header.extend_from_slice(&new_kek_nonce);
        new_header.extend_from_slice(&new_encrypted_cek);
        new_header.extend_from_slice(&new_encrypted_cek_nonce);
    }
    new_header.extend_from_slice(encrypted_verify_block);
    new_header.extend_from_slice(verify_block_hmac);
    new_header.extend_from_slice(original_file_hmac);
    new_header.write_u32::<BigEndian>(metadata_len as u32)?;
    new_header.extend_from_slice(&metadata);

    let old_header_len = (6 + old_num_passwords * ENTRY_SIZE + VERIFY_BLOCK_SIZE + 32 + 32 + 4 + metadata_len) as u64;
    let tmp_path = format!("{}.tmp", encrypted_path);
    let mut tmp_file = File::create(&tmp_path).context("创建临时文件失败")?;
    tmp_file.write_all(&new_header).context("写入新头部失败")?;
    file.seek(SeekFrom::Start(old_header_len)).context("设置文件指针失败")?;
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        tmp_file.write_all(&buffer[..n])?;
    }
    tmp_file.flush()?;
    rename(&tmp_path, encrypted_path).context("替换文件失败")?;
    Ok(())
}