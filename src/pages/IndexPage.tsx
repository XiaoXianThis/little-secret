import {route} from "preact-router"



export default function IndexPage() {

    return (
      <div class="w-full h-dvh flex">
          <div className="flex flex-row m-auto gap-4">

              <div onClick={() => route("/encrypt")}
                   className="cursor-pointer hover:scale-105 active:scale-95 select-none duration-150 w-56 h-32 p-2  border-1 rounded-2xl flex flex-col items-center justify-center gap-2">
                  <div>
                      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none"
                           stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                           className="feather feather-key">
                          <path
                            d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"></path>
                      </svg>
                  </div>
                  <div className="text-xl">加密解密</div>
                  <div className="text-xs text-center opacity-50">选择文件(夹)并加密</div>
              </div>

            <div onClick={() => route("/browse")}
                 className="cursor-pointer hover:scale-105 active:scale-95 select-none duration-150 w-56 h-32 p-2  border-1 rounded-2xl flex flex-col items-center justify-center gap-2">
              <div>
                <svg width="24px" height="24px" stroke-width="1.5" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M16 12H17.4C17.7314 12 18 12.2686 18 12.6V19.4C18 19.7314 17.7314 20 17.4 20H6.6C6.26863 20 6 19.7314 6 19.4V12.6C6 12.2686 6.26863 12 6.6 12H8M16 12V8C16 6.66667 15.2 4 12 4C8.8 4 8 6.66667 8 8V12M16 12H8" stroke="#000000" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path></svg>
              </div>
              <div className="text-xl">文件浏览</div>
              <div className="text-xs text-center opacity-50">浏览已加密文件</div>
            </div>

          </div>
      </div>
    )
}