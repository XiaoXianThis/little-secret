mod utils;

use std::{fs};
use crate::utils::encrypt::{decrypt_file, encrypt_file, reset_passwords};
use crate::utils::cry_info::{print_header_info, update_metadata};
use crate::utils::folder::{decrypt_file_temp, decrypt_folder, encrypt_folder, read_file_metadata, read_folder, read_img_base64};
use crate::utils::thumbnail::{make_thumbnail};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn test() -> String {
    // let result = encrypt_file("/Users/mac/Desktop/IMGS/1.jpg", "/Users/mac/Desktop/IMGS/1.jpg.cry", &[String::from("112233")], Some(b"my metadada".as_ref())).unwrap();
    // reset_passwords("/Users/mac/Desktop/IMGS/1.jpg.cry", "112233", &["aabbcc".to_string()]);
    // let result = decrypt_file("/Users/mac/Desktop/IMGS/1.jpg.cry", "/Users/mac/Desktop/IMGS/1.jpg.cry.jpg", "aabbcc").unwrap();
    // println!("{:?}", result);
    // update_metadata("/Users/mac/Desktop/IMGS/1.jpg.cry", b"this is new metadata!!!!".to_vec());
    print_header_info("/Users/mac/Desktop/IMGS/1.jpg.cry");

    format!("Hello! You've been greeted from Rust!")
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        // 注册 JS 可调用的函数
        .invoke_handler(tauri::generate_handler![
            test,
            encrypt_folder,         // 递归解密文件夹
            decrypt_folder,         // 递归加密文件夹
            read_folder,            // 读取文件夹中的全部文件
            read_file_metadata,     // 读取文件元数据
            decrypt_file_temp,      // 解密文件到临时文件
            read_img_base64,        // 将图片解码为base64（注意是图片而不是加密文件）
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}





