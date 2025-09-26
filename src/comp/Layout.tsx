import type {JSX} from "preact";
import { useRef} from "preact/hooks";



export default function Layout({children, class: className, title} : {
  children?: JSX.Element | JSX.Element[] | string
  class?: string
  title?: JSX.Element | JSX.Element[] | string
}) {

  const child = useRef<HTMLDivElement>(null)


  return (
    <div class={`h-dvh max-w-dvw overflow-y-scroll pt-4`}>
      <div className="fixed w-full md:max-w-[920px] left-1/2 -translate-x-1/2 px-2">
        <div className="absolute w-full h-full top-0 left-0 flex -z-10">
          <div className="m-auto font-bold">{title}</div>
        </div>
        {/*返回按钮*/}
        <div className=" select-none top-0 h-16 flex flex-row items-center">
          <div
            onClick={() => history.back()}
            className="w-14 h-14 rounded-full border-1 flex items-center justify-center cursor-pointer hover:scale-[102%]">
            <svg class="w-7 h-7" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M7.82843 10.9999H20V12.9999H7.82843L13.1924 18.3638L11.7782 19.778L4 11.9999L11.7782 4.22168L13.1924 5.63589L7.82843 10.9999Z"></path>
            </svg>
          </div>
        </div>

      </div>
      <div class={`pt-20 ${className} md:max-w-[900px] mx-auto`} ref={child}>
        {children}
      </div>
    </div>
  )
}

