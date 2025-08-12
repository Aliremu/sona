import * as React from "react"
import {
  Menubar,
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarTrigger,
} from "@/components/ui/menubar"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { platform } from "@tauri-apps/plugin-os"

export function Titlebar({
  children
}: React.ComponentProps<"div">) {
  const [isMacOS, setIsMacOS] = React.useState(false);

  React.useEffect(() => {
    const currentPlatform = platform();
    setIsMacOS(currentPlatform === 'macos');
  }, []);

  React.useEffect(() => {
    const appWindow = getCurrentWindow();

    document
      .getElementById('titlebar-minimize')
      ?.addEventListener('click', () => {
        appWindow.minimize();
      });
    document
      .getElementById('titlebar-maximize')
      ?.addEventListener('click', () => appWindow.toggleMaximize());
    document
      .getElementById('titlebar-close')
      ?.addEventListener('click', () => appWindow.close());
  });



  return (
    <div className="h-full flex flex-col">
      <div className="relative top-0 z-500 flex items-center bg-background border-b h-9" data-tauri-drag-region={isMacOS ? "true" : undefined}>
        {!isMacOS && (
          <>
          <Menubar className="border-none bg-transparent">
            <MenubarMenu>
              <MenubarTrigger>File</MenubarTrigger>
              <MenubarContent>
                <MenubarItem>
                  New Tab <MenubarShortcut>⌘T</MenubarShortcut>
                </MenubarItem>
                <MenubarItem>New Window</MenubarItem>
                <MenubarSeparator />
                <MenubarItem>Share</MenubarItem>
                <MenubarSeparator />
                <MenubarItem>Print</MenubarItem>
              </MenubarContent>
            </MenubarMenu>
            <MenubarMenu>
              <MenubarTrigger>Edit</MenubarTrigger>
              <MenubarContent>
                <MenubarItem>
                  New Tab <MenubarShortcut>⌘T</MenubarShortcut>
                </MenubarItem>
                <MenubarItem>New Window</MenubarItem>
                <MenubarSeparator />
                <MenubarItem>Share</MenubarItem>
                <MenubarSeparator />
                <MenubarItem>Print</MenubarItem>
              </MenubarContent>
            </MenubarMenu>
          </Menubar>
          <div className="flex-1 h-full" data-tauri-drag-region></div>
          <div className="flex items-center gap-2 h-full">
            <button id="titlebar-minimize" className="hover:bg-gray-200 px-2 h-full transition-[background-color] duration-100 ease-linear"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
              <path fill="currentColor" d="M19 13H5v-2h14z" />
            </svg></button>
            <button id="titlebar-maximize" className="hover:bg-gray-200 px-2 h-full transition-[background-color] duration-100 ease-linear"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
              <path fill="currentColor" d="M4 4h16v16H4zm2 4v10h12V8z" />
            </svg></button>
            <button id="titlebar-close" className="hover:bg-red-500 hover:text-white px-2 h-full transition-[background-color] duration-100 ease-linear"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
              <path
                fill="currentColor"
                d="M13.46 12L19 17.54V19h-1.46L12 13.46L6.46 19H5v-1.46L10.54 12L5 6.46V5h1.46L12 10.54L17.54 5H19v1.46z"
              />
            </svg></button>
          </div>
        </>
      )}
      </div>
      <div className="h-[calc(100vh-2.25rem)] w-full overflow-hidden flex flex-col">
        {children}
      </div>
    </div>
  )
}