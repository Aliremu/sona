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
import { Minus, Square, Copy, X } from "lucide-react"

export function Titlebar({
  children
}: React.ComponentProps<"div">) {
  const [isMacOS, setIsMacOS] = React.useState(false);
  const [isMaximized, setIsMaximized] = React.useState(false);

  React.useEffect(() => {
    const currentPlatform = platform();
    setIsMacOS(currentPlatform === 'macos');
  }, []);

  React.useEffect(() => {
    const appWindow = getCurrentWindow();

    // Check initial maximized state
    appWindow.isMaximized().then(setIsMaximized);

    // Listen for window resize events
    const unlisten = appWindow.listen('tauri://resize', async () => {
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    });

    document
      .getElementById('titlebar-minimize')
      ?.addEventListener('click', () => {
        appWindow.minimize();
      });
    document
      .getElementById('titlebar-maximize')
      ?.addEventListener('click', async () => {
        await appWindow.toggleMaximize();
        const maximized = await appWindow.isMaximized();
        setIsMaximized(maximized);
      });
    document
      .getElementById('titlebar-close')
      ?.addEventListener('click', () => appWindow.close());

    return () => {
      unlisten.then(f => f());
    };
  }, []);



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
            <button id="titlebar-minimize" className="hover:bg-gray-200 px-2 h-full transition-[background-color] duration-100 ease-linear">
              <Minus size={14} />
            </button>
            <button id="titlebar-maximize" className="hover:bg-gray-200 px-2 h-full transition-[background-color] duration-100 ease-linear">
              {isMaximized ? <Copy size={14} style={{ transform: 'scaleX(-1)' }} /> : <Square size={14} />}
            </button>
            <button id="titlebar-close" className="hover:bg-red-500 hover:text-white px-2 h-full transition-[background-color] duration-100 ease-linear">
              <X size={14} />
            </button>
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