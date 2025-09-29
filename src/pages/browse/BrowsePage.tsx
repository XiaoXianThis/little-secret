import Layout from "../../comp/Layout.tsx";
import {useEffect, useRef, useState} from "preact/hooks";
import { open } from "@tauri-apps/plugin-dialog"
import { h } from 'preact';
import compat from 'preact/compat';
import {List, RowComponentProps} from "react-window";
import {invoke} from "@tauri-apps/api/core";
import file from "../../assets/lock_file.png"
import {path} from "@tauri-apps/api";
import {useVirtualizer} from "@tanstack/react-virtual";
import { convertFileSrc } from '@tauri-apps/api/core';

type FileItem = { path:string, is_dir: boolean }

export default function () {

  const [folderPath, setFolderPath] = useState("");
  // 文件夹中所有文件
  const [folderFiles, setFolderFiles] = useState<FileItem[]>([])
  // 选中的文件
  const [activeItem, setActiveItem] = useState<FileItem>()
  // 详情大图
  const [previewSrc, setPreviewSrc] = useState("")

  useEffect(() => {
    openFolder()
  }, [])

  async function openFolder() {
    let path = await open({
      directory: true,
    })
    console.log(path)
    if (path) {
      setFolderPath(path)
      readFolder(path)
    }
  }

  // 读取文件夹中的文件
  async function readFolder(path:string) {
    let folderFiles = await invoke("read_folder", { path }) as string;
    let files = JSON.parse(folderFiles) as FileItem[];
    setFolderFiles(files.filter( item => item.path.endsWith(".cry") && !item.path.includes(".DS_Store") ));
  }

  // 虚拟滚动加载
  const parentRef = useRef<HTMLDivElement>(null)
  const virtualizer = useVirtualizer({
    count: folderFiles.length,
    getScrollElement: () => parentRef.current, // 滚动容器
    estimateSize: () => 110
  })

  useEffect(() => {
    if (activeItem) {
      // alert(activeItem.path)
      decrypt_file_temp(activeItem.path)
    }
  }, [activeItem]);
  // 解密文件到临时文件
  async function decrypt_file_temp(path: string) {
    let result_path = await invoke("decrypt_file_temp", { path, password: "112233" }) as string
    const url = convertFileSrc(result_path)
    setPreviewSrc(url + `?t=${Date.now()}`) //加上时间戳，强制更新
  }

  return (
    <div class="w-dvw h-dvh overflow-hidden flex flex-row p-2 bg-black">

      {/*左侧浏览视图*/}
      <div className=" top-0 left-0 h-full w-[180px] items-center overflow-hidden flex flex-col relative">

        {/*返回按钮*/}
        <div className="z-50 w-full">
          <div className=" select-none top-0 h-16 flex flex-row items-center pl-2 text-white">
            <div
              onClick={() => history.back()}
              className="backdrop-blur-lg w-14 h-14 border-white rounded-full border-1 flex items-center justify-center cursor-pointer duration-150 active:scale-95 hover:scale-[105%]">
              <svg className="w-7 h-7" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M7.82843 10.9999H20V12.9999H7.82843L13.1924 18.3638L11.7782 19.778L4 11.9999L11.7782 4.22168L13.1924 5.63589L7.82843 10.9999Z"></path>
              </svg>
            </div>
          </div>
        </div>

        <div ref={parentRef} class="pt-4 flex flex-col items-center flex-1 h-0 overflow-y-scroll w-full">
          <div className="z-0" style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: '150px',
            position: 'relative',
          }}>
            {virtualizer.getVirtualItems().map((virtualItem, index) => (
              <div
                key={virtualItem.key}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  height: `${virtualItem.size}px`,
                  transform: `translateY(${virtualItem.start}px)`,
                }}
              >
                <RowComp item={folderFiles[virtualItem.index]} onClick={(metadata) => {
                  setPreviewSrc(`data:image/*;base64,${metadata.thumbnail}`)
                  setTimeout(() => {
                    setActiveItem(folderFiles[virtualItem.index])
                  }, 1000)
                }}/>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/*右侧预览视图*/}
      <div className="h-full sticky right-0 top-0 w-0 flex-1 flex flex-col items-center justify-center px-4">
        { previewSrc && <img src={`${previewSrc}`} className="h-full w-full object-contain"/> }
      </div>

    </div>
  )
}


type MetaData = {
  thumbnail: string;
}

function RowComp({item, onClick}: { item: FileItem, onClick:(metadata:MetaData)=>void }) {

  const [metadata, setMetadata] = useState<MetaData>()
  const [text, setText] = useState("")

  useEffect(() => {
    load_metadata()
  }, []);

  // 读取元数据
  async function load_metadata() {
    if (!item.is_dir) {
      let result = await invoke("read_file_metadata", {path: item.path}) as string;
      let metadata = JSON.parse(result) as MetaData;
      setMetadata(metadata);
    }
  }

  return (
    metadata?.thumbnail && (
      <div onClick={() => onClick(metadata)} className="relative  w-[150px] max-w-[150px] cursor-pointer group">
        <div class="text-white">{text}</div>
        <img src={`data:image/jpeg;base64,${metadata.thumbnail}`}
             className="w-full h-[100px] object-contain hover:scale-105 active:scale-95 duration-150 p-1"/>
      </div>
    )
  )
}


