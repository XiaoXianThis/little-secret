mod utils;

use std::{fs};
use crate::utils::encrypt::{decrypt_file, encrypt_file, reset_passwords};
use crate::utils::cry_info::{print_header_info, update_metadata};


// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn test(name: &str) -> String {

    // let result = encrypt_file("/Users/mac/Desktop/IMGS/1.jpg", "/Users/mac/Desktop/IMGS/1.jpg.cry", &[String::from("112233")], Some(b"my metadada".as_ref())).unwrap();
    // reset_passwords("/Users/mac/Desktop/IMGS/1.jpg.cry", "112233", &["aabbcc".to_string()]);
    // let result = decrypt_file("/Users/mac/Desktop/IMGS/1.jpg.cry", "/Users/mac/Desktop/IMGS/1.jpg.cry.jpg", "aabbcc").unwrap();
    // println!("{:?}", result);
    // update_metadata("/Users/mac/Desktop/IMGS/1.jpg.cry", b"this is new metadata!!!!".to_vec());
    // print_header_info("/Users/mac/Desktop/IMGS/1.jpg.cry");


    format!("Hello, {}! You've been greeted from Rust!", name)
}

//
#[tauri::command]
fn encrypt_folder(path: &str, passwords: Vec<&str>) -> String {
    if path.is_empty() { return "加密失败: 路径为空！".to_string(); }
    let result = fs::read_dir(path);
    if result.is_err() { return format!("打开路径失败({})", path) }
    // 遍历传入的文件夹
    for entry in result.unwrap() {
        // 读取单个路径
        let entry = entry.unwrap();
        let path = entry.path().to_string_lossy().to_string();
        // 不处理文件夹和已加密的文件
        if !entry.path().is_dir() && !path.ends_with(".cry") {
            let vec_passwords: Vec<String> = passwords.iter().map(|s| s.to_string()).collect();
            encrypt_file(&path, &format!("{}.cry", path).to_string(), &vec_passwords, None).unwrap();
            // 删除原文件
            fs::remove_file(entry.path()).unwrap();
        } else if entry.path().is_dir() {
            encrypt_folder(path.as_str(), passwords.clone());
        }
        // encrypt_file(entry.path()).unwrap()
    }

    return format!("加密完毕：{}，密码：{:?}", path, passwords)
}

#[tauri::command]
fn decrypt_folder(path: &str, password: &str) -> String {
    if path.is_empty() { return "加密失败: 路径为空！".to_string(); }
    let result = fs::read_dir(path);
    if result.is_err() { return format!("打开路径失败({})", path) }
    // 遍历传入的文件夹
    for entry in result.unwrap() {
        // 读取单个路径
        let entry = entry.unwrap();
        let path = entry.path().to_string_lossy().to_string();
        // 只处理.cry文件
        if !entry.path().is_dir() && path.ends_with(".cry") {
            let result = decrypt_file(&path, format!("{}", path.split_at(path.len()-4).0).as_str(), password).unwrap();
            // 解密成功
            if result.0 {
                fs::remove_file(entry.path()).unwrap();
            }
            else {
                return format!("解密失败：({})", path)
            }
        }
    }
    return "解密完毕".to_string();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            test,
            encrypt_folder,
            decrypt_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}





