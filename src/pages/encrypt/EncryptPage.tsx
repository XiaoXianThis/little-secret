import Layout from "../../comp/Layout.tsx";
import {useState} from "preact/hooks";
import { open } from "@tauri-apps/plugin-dialog"
import {invoke} from "@tauri-apps/api/core";


export default function EncryptPage() {

  const [path, setPath] = useState("")
  const [password, setPassword] = useState("")
  const [password2, setPassword2] = useState("")
  const [password_dec, setPasswordDec] = useState("")
  const [dirFiles, setDirFiles] = useState<string[]>([])
  const [loading, setLoading] = useState(false)

  // 选择路径
  async function select_path() {
    let path = await open({
      directory: true,
    })
    if (path) {
      setPath(path)
    }
  }

  async function encrypt() {

    let passwords:string[] = []
    // 检查密码
    if (password.length >= 6) passwords.push(password); else return alert("主密码最少 6 位")
    if (password2.length >= 6) passwords.push(password2); else return alert("备用码最少 6 位")

    setLoading(true)
    let result = await invoke("encrypt_folder", { path, passwords })
    setLoading(false)
    alert(result)
  }

  async function decrypt() {
    setLoading(true)
    let result = await invoke("decrypt_folder", {path, password: password_dec})
    setLoading(false)
    alert(result)
  }

  // 读取文件夹内容
  async function readDir(path: string) {
    let result : string[] = await (await fetch("/api/read_path", {
      method: "POST",
      body: path
    })).json()
    setDirFiles(result.filter(item => item.endsWith(".cry")))
  }


  return (
    <Layout class="min-h-dvh flex flex-col gap-6 px-12" title="加密解密">
      <div className="grid grid-cols-2 gap-6 ">
        <div className="w-full h-48 border-1 rounded-2xl flex flex-col px-6 pr-2 relative py-4 col-span-2">
          <div class="font-bold mb-2">文件夹</div>
          <div className={`${path ? "" : "opacity-40"}`}>{path ? path : "选择要加密的文件/夹路径"}</div>
          <div class="text-sm mt-3 opacity-50">已加密 {dirFiles.length} 个文件</div>
          <div onClick={() => !loading && select_path()}
               className={`px-6 absolute right-2 bottom-2 border-1 py-2.5 ${ !loading ? "bg-black" : "bg-black/50" } text-white rounded-2xl select-none hover:scale-105 active:scale-95 duration-150 cursor-pointer`}>选择
          </div>
          {loading && (
            <div className="absolute right-6 text-sm flex flex-row items-center gap-2s">
              <div className="animate-spin w-6 h-6">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
                  <path
                    d="M18.364 5.63604L16.9497 7.05025C15.683 5.7835 13.933 5 12 5C8.13401 5 5 8.13401 5 12C5 15.866 8.13401 19 12 19C15.866 19 19 15.866 19 12H21C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C14.4853 3 16.7353 4.00736 18.364 5.63604Z"></path>
                </svg>
              </div>
              <div>执行中</div>
            </div>
          )}
        </div>

        <div className=" relative w-full h-62 border-1 rounded-2xl flex flex-col  pl-6 pr-6  gap-2 py-4">
          <div class="font-bold mb-2">加密</div>
          <input className="h-12 outline-0 bg-black/5 px-4 rounded-xl" placeholder="输入密码"
                 type="text" onChange={event => {
            setPassword((event.target! as HTMLInputElement).value)
          }} value={password}/>

          <input className="h-12 outline-0 bg-black/5 px-4 rounded-xl" placeholder="备用密码" type="text"
                 onChange={event => {
                   setPassword2((event.target! as HTMLInputElement).value)
                 }} value={password2}/>

          <div className="flex flex-row gap-1 absolute right-2 bottom-2">
            <div onClick={() => path && password.length >= 6  && password2.length >= 6 &&  !loading && encrypt()}
                 className={`px-6 border-1 py-2.5 ${path && password.length >= 6 && password2.length >= 6 &&  !loading ? "bg-red-500 cursor-pointer  hover:scale-105 active:scale-95" : "bg-red-300 "} text-white rounded-2xl select-none duration-150 `}>加密
            </div>
          </div>
        </div>

        <div className=" relative w-full h-62 border-1 rounded-2xl flex flex-col  pl-6 pr-6  gap-2 py-4">
          <div class="font-bold mb-2">解密</div>
          <div class="text-sm opacity-50">使用主密码 或者 备用密码解密</div>
          <input className="h-12 outline-0 bg-black/5 px-4 rounded-xl" placeholder="输入密码"
                 type="text" onChange={event => {
            setPasswordDec((event.target! as HTMLInputElement).value)
          }} value={password_dec}/>

          <div className="flex flex-row gap-1 absolute right-2 bottom-2">
            <div onClick={() => path && password_dec.length >= 6 && !loading && decrypt()}
                 className={`px-6 border-1 py-2.5 ${path && password_dec.length >= 6 && !loading ? "bg-blue-500 cursor-pointer  hover:scale-105 active:scale-95" : "bg-blue-300 "} text-white rounded-2xl select-none duration-150 `}>解密
            </div>
          </div>
        </div>
      </div>
    </Layout>
  )
}

