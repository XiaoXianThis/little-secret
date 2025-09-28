use std::io::Cursor;
use image::imageops::{overlay, FilterType};
use image::{open, DynamicImage, GenericImageView, ImageFormat};
use base64::{engine::general_purpose, Engine};


// 制作缩略图，传入路径，返回base64
pub fn make_thumbnail(path: &String) -> String {
    // 尝试作为图片打开
    let result = open(path);
    let mut buffer: &[u8];
    // 不是图片，返回默认预览图
    if result.is_err() {
        // 使用预制图标
        let img = open("resources/lock_file.png").unwrap();
        // img.save(path).unwrap();
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
        let resized = img.resize(100, 100, FilterType::Nearest);
        let blurred = resized.blur(20.0);
        // let lock = open("resources/lock.png").unwrap();
        // overlay(&mut blurred, &lock, 0, 0);
        // blurred.save(path).unwrap();
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        blurred.write_to(&mut cursor, ImageFormat::Jpeg).unwrap();
        // 转为base64
        return general_purpose::STANDARD.encode(buffer);
    }

}

