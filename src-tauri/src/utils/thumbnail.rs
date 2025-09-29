use std::fs::File;
use std::io::{BufReader, Cursor};
use image::imageops::{overlay, FilterType};
use image::{load_from_memory, ImageFormat, ImageReader};
use base64::{engine::general_purpose, Engine};

// 嵌入图片 硬编码
const LOCK_FILE_H_PNG: &[u8] = include_bytes!("../../resources/lock_file_h.png");
const LOCK_PNG: &[u8] = include_bytes!("../../resources/lock.png");


// 制作缩略图，传入路径，返回base64
#[tauri::command]
pub fn make_thumbnail(path: &String) -> String {
    // 尝试作为图片打开
    // let result = open(path);
    let result = ImageReader::new(BufReader::new(File::open(path).unwrap())).with_guessed_format().unwrap().decode();
    let mut buffer: &[u8];
    // 不是图片，返回默认预览图
    if result.is_err() {
        let img = load_from_memory(LOCK_FILE_H_PNG).unwrap();
        // 使用预制图标
        // let img = open(app.path().resolve("lock_file_h.png", BaseDirectory::Resource).unwrap()).unwrap();
        // 3. 将图片保存到内存缓冲区（以 PNG 格式为例）
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageFormat::Jpeg).unwrap();
        // 转为base64
        return general_purpose::STANDARD.encode(buffer);
    }
    // 是图片，生成模糊预览图
    else {
        // 生成预览图
        let img = result.unwrap();
        let resized = img.resize(150, 150, FilterType::Nearest);
        let mut blurred = resized.blur(20.0);
        // let lock = open(app.path().resolve("lock.png", BaseDirectory::Resource).unwrap()).unwrap();
        let lock = load_from_memory(LOCK_PNG).unwrap();
        overlay(&mut blurred, &lock, 0, 0);
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        blurred.write_to(&mut cursor, ImageFormat::Jpeg).unwrap();
        // 转为base64
        return general_purpose::STANDARD.encode(buffer);
    }

}

