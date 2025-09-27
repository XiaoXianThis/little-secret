use image::imageops::FilterType;
use image::{open, GenericImageView};

// 制作缩略图

pub fn make_thumbnail(path: &String) {
    let img = open(path).unwrap();
    let resized = img.resize(100, 100, FilterType::Nearest);
    let blurred = resized.blur(10.0);
    blurred.save(path).unwrap();
}

