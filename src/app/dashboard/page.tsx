import { useState, useRef, useEffect } from "react"
import { useTheme } from "next-themes"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Button } from "@/components/ui/button"
import { Switch } from "@/components/ui/switch"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { ScrollArea } from "@/components/ui/scroll-area"
import { SheetMusicViewer } from "@/components/sheet-music-viewer"
import { AudioVisualizer } from "@/components/audio-visualizer"
import { VideoPlayer } from "@/components/video-player"
import { PlaylistSidebar } from "@/components/playlist-sidebar"
import { VstDock } from "@/components/vst-dock"
import { UnifiedControls } from "@/components/unified-controls"
import { SongInfoModal } from "@/components/song-info-modal"
import { PracticeStatsModal } from "@/components/practice-stats-modal"
import { SettingsModal } from "@/components/settings-modal"
import { Play, FileMusic, Info, BarChart3, Settings, AudioWaveform, Sun, Moon, Plus } from "lucide-react"
import { Titlebar } from "@/components/title-bar"
import { invoke } from "@tauri-apps/api/core"

// Sample data structure for playlists
const initialPlaylists = [
  {
    id: 1,
    name: "ZUTOMAYO",
    songs: [
      {
        id: 1,
        title: "隣室上のクラッカー",
        subtitle: "Guitar Cover with TAB",
        videoUrl: "https://www.youtube.com/embed/ltBHhvejvE8",
        sheetMusicUrl: "/guitar-sheet-music.png",
        bpm: 120,
        duration: "3:15",
      },
      {
        id: 2,
        title: "Saturn",
        subtitle: "Lead Guitar",
        videoUrl: "https://www.youtube.com/embed/Q9WZtxRWieM",
        sheetMusicUrl: "/guitar-sheet-music.png",
        bpm: 140,
        duration: "4:10",
      },
    ],
  },
  {
    id: 2,
    name: "Yorushika",
    songs: [
      {
        id: 3,
        title: "だから僕は音楽を辞めた",
        subtitle: "Acoustic Guitar",
        videoUrl: "https://www.youtube.com/embed/KTZ-y85Erus",
        sheetMusicUrl: "/guitar-sheet-music.png",
        bpm: 95,
        duration: "4:32",
      },
    ],
  },
  {
    id: 3,
    name: "Yoasobi",
    songs: [
      {
        id: 4,
        title: "夜に駆ける",
        subtitle: "Guitar Arrangement",
        videoUrl: "https://www.youtube.com/embed/x8VYWazR5mE",
        sheetMusicUrl: "/guitar-sheet-music.png",
        bpm: 130,
        duration: "4:23",
      },
    ],
  },
]

export default function MusicPracticeApp() {
  const { theme, setTheme } = useTheme()
  const [isPlaying, setIsPlaying] = useState(false)
  const [showVisualizer, setShowVisualizer] = useState(false)
  const [currentTime, setCurrentTime] = useState(0)
  const duration = 195
  const [volume, setVolume] = useState(75)
  const [bpm, setBpm] = useState(120)
  const [playbackRate, setPlaybackRate] = useState(1)
  const [pitch, setPitch] = useState(0)
  const [loopStart, setLoopStart] = useState("")
  const [loopEnd, setLoopEnd] = useState("")
  const [isLooping, setIsLooping] = useState(false)
  const [plugins, setPlugins] = useState<{ id: number; name: string; enabled: boolean; type: string; color: string }[]>([])
  const [page, setPage] = useState(1)
  const [playlists, setPlaylists] = useState(initialPlaylists)
  const [currentSong, setCurrentSong] = useState(initialPlaylists[0].songs[0])
  const [currentArtist, setCurrentArtist] = useState(initialPlaylists[0].name)
  const [vstDockExpanded, setVstDockExpanded] = useState(false)
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false)
  const [songInfoOpen, setSongInfoOpen] = useState(false)
  const [practiceStatsOpen, setPracticeStatsOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [activeTab, setActiveTab] = useState("video")
  const videoRef = useRef<HTMLVideoElement>(null)

  type PluginInfo = {
    id: number
    name: string
  }

  const [discoveredPlugins, setDiscoveredPlugins] = useState<string[]>([])
  const [showPluginDialog, setShowPluginDialog] = useState(false)

  useEffect(() => {
    async function run() {
      const response: PluginInfo[] = await invoke("get_loaded_plugins");
      let plugins = [];
      for (const plugin of response) {
        plugins.push({
          id: plugin.id,
          name: plugin.name,
          enabled: true,
          type: "Reverb",
          color: "red",
        });
      }

      setPlugins(plugins);

      // Also load discovered plugins
      const discovered: string[] = await invoke("get_discovered_plugins");
      setDiscoveredPlugins(discovered);
    }

    run();
  }, [])

  const toggleVisualizer = () => setShowVisualizer(!showVisualizer)

  const togglePlugin = (id: number) => {
    setPlugins(plugins.map((plugin) => (plugin.id === id ? { ...plugin, enabled: !plugin.enabled } : plugin)))
  }

  const addPlugin = () => {
    setShowPluginDialog(true)
  }

  const loadPlugin = async (pluginPath: string) => {
    try {
      await invoke("load_plugin", { path: pluginPath });
      // Refresh the loaded plugins list
      const response: PluginInfo[] = await invoke("get_loaded_plugins");
      let plugins = [];
      for (const plugin of response) {
        plugins.push({
          id: plugin.id,
          name: plugin.name,
          enabled: true,
          type: "VST",
          color: "blue",
        });
      }
      setPlugins(plugins);
      setShowPluginDialog(false);
    } catch (error) {
      console.error("Failed to load plugin:", error);
    }
  }

  const removePlugin = (id: number) => {
    invoke("remove_plugin", { pluginId: id }).catch((error) => {
      console.error("Failed to remove plugin:", error);
    });
    setPlugins(plugins.filter((plugin) => plugin.id !== id))
  }

  const openPluginEditor = async (id: number) => {
    try {
      await invoke("open_plugin_editor", { pluginId: id });
    } catch (error) {
      console.error("Failed to open plugin editor:", error);
    }
  }

  const handlePageChange = (newPage: number) => {
    setPage(newPage)
  }

  const handleSongSelect = (song: any, artistName: string) => {
    setCurrentSong(song)
    setCurrentArtist(artistName)
    setBpm(song.bpm)
    setIsPlaying(false)
    setPage(1)
    setCurrentTime(0)
    setLoopStart("")
    setLoopEnd("")
    setIsLooping(false)
  }

  const addNewPlaylist = (name: string) => {
    const newPlaylist = {
      id: playlists.length + 1,
      name,
      songs: [],
    }
    setPlaylists([...playlists, newPlaylist])
  }

  const addSongToPlaylist = (playlistId: number, song: any) => {
    setPlaylists(
      playlists.map((playlist) =>
        playlist.id === playlistId ? { ...playlist, songs: [...playlist.songs, song] } : playlist,
      ),
    )
  }

  const controlsHeight = vstDockExpanded ? "350px" : "140px"

  return (
    <Titlebar>
    <div className="flex h-full bg-gradient-to-br from-background via-background to-muted/20">
      {/* Playlist Sidebar */}
      <PlaylistSidebar
        playlists={playlists}
        currentSong={currentSong}
        onSongSelect={handleSongSelect}
        onAddPlaylist={addNewPlaylist}
        onAddSong={addSongToPlaylist}
        isCollapsed={sidebarCollapsed}
        onToggleCollapse={() => setSidebarCollapsed(!sidebarCollapsed)}
      >
        <VstDock
          plugins={plugins}
          expanded={vstDockExpanded}
            sidebarCollapsed={sidebarCollapsed}
            onToggleExpanded={() => setVstDockExpanded(!vstDockExpanded)}
            onTogglePlugin={togglePlugin}
            onAddPlugin={addPlugin}
            onRemovePlugin={removePlugin}
            onOpenPluginEditor={openPluginEditor}
            onExpandSidebar={() => setSidebarCollapsed(false)}
        />
      </PlaylistSidebar>

      {/* Plugin Selection Dialog */}
      <Dialog open={showPluginDialog} onOpenChange={setShowPluginDialog}>
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>Add VST Plugin</DialogTitle>
          </DialogHeader>
          <div className="py-4">
            <p className="text-sm text-muted-foreground mb-4">
              Select a plugin to add to your rack:
            </p>
            <ScrollArea className="h-[300px] w-full rounded-md border p-4">
              <div className="space-y-2">
                {discoveredPlugins.map((pluginPath, index) => {
                  const pluginName = pluginPath.split(/[/\\]/).pop()?.replace(/\.(dll|vst3)$/i, '') || pluginPath;
                  return (
                    <Button
                      key={index}
                      variant="ghost"
                      className="w-full justify-start h-auto p-4 border border-border/50 hover:border-border hover:bg-accent/50"
                      onClick={() => loadPlugin(pluginPath)}
                    >
                      <div className="flex items-center gap-3">
                        <div className="h-8 w-8 rounded bg-primary/10 flex items-center justify-center">
                          <Plus className="h-4 w-4 text-primary" />
                        </div>
                        <div className="text-left">
                          <div className="font-medium">{pluginName}</div>
                          <div className="text-xs text-muted-foreground truncate max-w-[280px]">
                            {pluginPath}
                          </div>
                        </div>
                      </div>
                    </Button>
                  );
                })}
                {discoveredPlugins.length === 0 && (
                  <div className="text-center py-8 text-muted-foreground">
                    <p>No plugins discovered.</p>
                    <p className="text-xs mt-2">
                      Check your plugin paths in settings and scan for plugins.
                    </p>
                  </div>
                )}
              </div>
            </ScrollArea>
          </div>
        </DialogContent>
      </Dialog>

      {/* Main Content */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <header className="h-16 border-b border-border/50 bg-background/80 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60">
          <div className="flex h-full items-center px-8">
            <div className="flex items-center gap-4">
              <div className="h-9 w-9 rounded-xl bg-gradient-to-br from-accent to-accent/80 flex items-center justify-center shadow-lg shadow-accent/25">
                <div className="h-4 w-4 rounded-full bg-white/90" />
              </div>
              <div className="flex flex-col">
                <h1 className="text-xl font-bold tracking-tight bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text text-transparent">
                  Sona
                </h1>
                {/* <Badge
                  variant="secondary"
                  className="text-xs bg-accent/15 text-accent border-accent/20 font-medium w-fit"
                >
                  Pro
                </Badge> */}
              </div>
            </div>
            <div className="ml-auto flex items-center gap-3">
              {/* Dark Mode Toggle */}
              <Button
                variant="ghost"
                size="icon"
                className="h-9 w-9 hover:bg-accent/20 hover:text-accent transition-all duration-200"
                onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
              >
                <Sun className="h-4 w-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
                <Moon className="absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
                <span className="sr-only">Toggle theme</span>
              </Button>

              {/* Visualizer Toggle */}
              <div className="flex items-center gap-3 mr-6 px-3 py-2 rounded-lg bg-muted/50 border border-border/50">
                <Switch
                  id="visualizer-toggle"
                  checked={showVisualizer}
                  onCheckedChange={toggleVisualizer}
                  className="data-[state=checked]:bg-accent"
                />
                <Label htmlFor="visualizer-toggle" className="text-sm flex items-center gap-2 font-medium">
                  <AudioWaveform className="h-4 w-4 text-accent" />
                  Visualizer
                </Label>
              </div>

              <div className="flex items-center gap-1 bg-muted/30 rounded-lg p-1">
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 hover:bg-accent/20 hover:text-accent transition-all duration-200"
                  onClick={() => setSongInfoOpen(true)}
                >
                  <Info className="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 hover:bg-accent/20 hover:text-accent transition-all duration-200"
                  onClick={() => setPracticeStatsOpen(true)}
                >
                  <BarChart3 className="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 hover:bg-accent/20 hover:text-accent transition-all duration-200"
                  onClick={() => setSettingsOpen(true)}
                >
                  <Settings className="h-4 w-4" />
                </Button>
              </div>

              <div className="flex items-center gap-3 px-4 py-2 rounded-lg bg-card border border-border/50 shadow-sm">
                <Badge variant="outline" className="text-xs border-accent/30 text-accent bg-accent/5 font-medium">
                  {currentArtist}
                </Badge>
                <div className="h-1 w-1 rounded-full bg-muted-foreground/50" />
                <span className="text-sm font-semibold text-foreground/90">{currentSong.title}</span>
              </div>
            </div>
          </div>
        </header>

        {/* Content Area */}
        <div className="flex-1 overflow-hidden" style={{ height: `calc(100vh - 16px - ${controlsHeight})` }}>
          <Tabs value={activeTab} onValueChange={setActiveTab} className="h-full flex flex-col">
            <div className="px-8 pt-4 pb-2">
              <TabsList className="h-10 w-fit bg-muted/30 backdrop-blur-sm border border-border/30 rounded-xl p-1 shadow-sm">
                <TabsTrigger
                  value="video"
                  className="h-8 px-4 rounded-lg text-sm font-medium transition-all duration-200 data-[state=active]:bg-accent data-[state=active]:text-accent-foreground data-[state=active]:shadow-sm hover:bg-muted/50"
                >
                  <Play className="h-3.5 w-3.5 mr-2" />
                  Video
                </TabsTrigger>
                <TabsTrigger
                  value="sheet"
                  className="h-8 px-4 rounded-lg text-sm font-medium transition-all duration-200 data-[state=active]:bg-accent data-[state=active]:text-accent-foreground data-[state=active]:shadow-sm hover:bg-muted/50"
                >
                  <FileMusic className="h-3.5 w-3.5 mr-2" />
                  Sheet Music
                </TabsTrigger>
              </TabsList>
            </div>

            {/* Video Playback Tab */}
            <TabsContent value="video" className="flex-1 overflow-hidden m-0" forceMount>
              <div className="h-full px-8 pb-8">
                {showVisualizer ? (
                  <div className="h-full rounded-2xl border border-border/50 bg-card/50 backdrop-blur-sm shadow-xl shadow-black/5">
                    <div className="p-6 border-b border-border/50">
                      <h3 className="font-bold text-lg flex items-center gap-3">
                        <div className="h-3 w-3 rounded-full bg-accent animate-pulse shadow-lg shadow-accent/50" />
                        Audio Visualizer
                      </h3>
                      <p className="text-sm text-muted-foreground mt-1 font-medium">Real-time frequency analysis</p>
                    </div>
                    <div className="p-6 h-[calc(100%-100px)]">
                      <AudioVisualizer videoElement={videoRef.current} isPlaying={isPlaying} />
                    </div>
                  </div>
                ) : (
                  <div className="h-full rounded-2xl overflow-hidden shadow-2xl shadow-black/10 border border-border/50">
                    <VideoPlayer
                      ref={videoRef}
                      currentSong={currentSong}
                      isPlaying={isPlaying}
                      currentTime={currentTime}
                      onTimeUpdate={setCurrentTime}
                    />
                  </div>
                )}
              </div>
            </TabsContent>

            {/* Sheet Music Tab */}
            <TabsContent value="sheet" className="flex-1 overflow-hidden m-0" forceMount>
              <div className="h-full px-8 pb-8">
                {showVisualizer ? (
                  <div className="h-full rounded-2xl border border-border/50 bg-card/50 backdrop-blur-sm shadow-xl shadow-black/5">
                    <div className="p-6 border-b border-border/50">
                      <h3 className="font-bold text-lg flex items-center gap-3">
                        <div className="h-3 w-3 rounded-full bg-accent shadow-lg shadow-accent/50" />
                        Audio Visualizer
                      </h3>
                      <p className="text-sm text-muted-foreground mt-1 font-medium">Real-time frequency analysis</p>
                    </div>
                    <div className="p-6 h-[calc(100%-100px)]">
                      <AudioVisualizer videoElement={videoRef.current} isPlaying={isPlaying} />
                    </div>
                  </div>
                ) : (
                  <div className="h-full rounded-2xl border border-border/50 bg-card/50 backdrop-blur-sm shadow-xl shadow-black/5 overflow-hidden">
                    <div className="p-6 border-b border-border/50">
                      <h3 className="font-bold text-lg flex items-center gap-3">
                        <div className="h-3 w-3 rounded-full bg-accent shadow-lg shadow-accent/50" />
                        Sheet Music
                      </h3>
                      <p className="text-sm text-muted-foreground mt-1 font-medium">
                        {currentArtist} - {currentSong.title}
                      </p>
                    </div>
                    <div className="h-[calc(100%-100px)]">
                      <SheetMusicViewer
                        sheetMusicUrl={currentSong.sheetMusicUrl}
                        currentPage={page}
                        onPageChange={handlePageChange}
                      />
                    </div>
                  </div>
                )}
              </div>
            </TabsContent>
          </Tabs>
        </div>

        {/* Unified Controls */}
        <UnifiedControls
          isPlaying={isPlaying}
          onPlayStateChange={setIsPlaying}
          currentTime={currentTime}
          duration={duration}
          onSeek={setCurrentTime}
          volume={volume}
          onVolumeChange={setVolume}
          bpm={bpm}
          onBpmChange={setBpm}
          playbackRate={playbackRate}
          onPlaybackRateChange={setPlaybackRate}
          pitch={pitch}
          onPitchChange={setPitch}
          loopStart={loopStart}
          onLoopStartChange={setLoopStart}
          loopEnd={loopEnd}
          onLoopEndChange={setLoopEnd}
          isLooping={isLooping}
          onLoopingChange={setIsLooping}
          currentSong={currentSong}
          showPageControls={activeTab === "sheet" && !showVisualizer}
          currentPage={page}
          onPageChange={handlePageChange}
        />
      </div>

      {/* Modals */}
      <SongInfoModal
        open={songInfoOpen}
        onOpenChange={setSongInfoOpen}
        currentSong={currentSong}
        currentArtist={currentArtist}
        bpm={bpm}
      />

      <PracticeStatsModal open={practiceStatsOpen} onOpenChange={setPracticeStatsOpen} />

      <SettingsModal open={settingsOpen} onOpenChange={setSettingsOpen} />
    </div>
    </Titlebar>
  )
}
