use std::fs;
use std::fs::metadata;
use image::imageops::thumbnail;
use rayon::ThreadPoolBuilder;
use serde::{Deserialize, Serialize};
use crate::utils::encrypt::{decrypt_file, encrypt_file};
use crate::utils::thumbnail::make_thumbnail;

#[derive(Serialize, Deserialize, Debug)]
// 加密文件的元数据
struct MetaData {
    thumbnail: String,
}

// 递归加密文件夹
#[tauri::command]
pub fn encrypt_folder(path: &str, passwords: Vec<&str>) -> String {
    if path.is_empty() { return "加密失败: 路径为空！".to_string(); }
    let result = fs::read_dir(path);
    if result.is_err() { return format!("打开路径失败({})", path) }
    // 创建线程池
    let pool = ThreadPoolBuilder::new().num_threads(16).build().unwrap();
    pool.scope(|scope|{
        // 遍历传入的文件夹
        for entry in result.unwrap() {
            // 分配到线程去做
            scope.spawn(|x| {
                // 读取单个路径
                let entry = entry.unwrap();
                let path = entry.path().to_string_lossy().to_string();
                // 不处理文件夹和已加密的文件
                if !entry.path().is_dir() && !path.ends_with(".cry") {
                    // 缩略图 base64
                    let thumbnail_base64 = make_thumbnail(&path);
                    println!("{}", thumbnail_base64);

                    // 构建文件头
                    let metadata = MetaData {
                        thumbnail: thumbnail_base64
                    };

                    // 加密
                    let vec_passwords: Vec<String> = passwords.iter().map(|s| s.to_string()).collect();
                    encrypt_file(
                        &path,
                        &format!("{}.cry", path).to_string(),
                        &vec_passwords,
                        Some(serde_json::to_string(&metadata).unwrap().as_bytes())
                    ).unwrap();

                    // 删除原文件
                    fs::remove_file(entry.path()).unwrap();

                } else if entry.path().is_dir() {
                    // 递归子文件夹
                    encrypt_folder(path.as_str(), passwords.clone());
                }
                // encrypt_file(entry.path()).unwrap()
            });
        }
    });

    return format!("加密完毕：{}，密码：{:?}", path, passwords)
}

// 递归解密文件夹
#[tauri::command]
pub fn decrypt_folder(path: &str, password: &str) -> String {
    if path.is_empty() { return "加密失败: 路径为空！".to_string(); }
    let result = fs::read_dir(path);
    if result.is_err() { return format!("打开路径失败({})", path) }
    // 创建线程池
    let pool = ThreadPoolBuilder::new().num_threads(16).build().unwrap();
    pool.scope(|scope| {
        // 遍历传入的文件夹
        for entry in result.unwrap() {
            // 分配到线程去做
            scope.spawn(|x| {
                // 读取单个路径
                let entry = entry.unwrap();
                let path = entry.path().to_string_lossy().to_string();
                // 只处理.cry文件
                if !entry.path().is_dir() && path.ends_with(".cry") {
                    let result = decrypt_file(&path, format!("{}", path.split_at(path.len()-4).0).as_str(), password);
                    // 解密成功
                    if result.is_ok() {
                        fs::remove_file(entry.path()).unwrap();
                    }
                    else {
                        return println!("解密失败！")
                    }
                }
                else if entry.path().is_dir() {
                    decrypt_folder(path.as_str(), password);
                }
            })
        }
    });
    return "解密完毕".to_string();
}
