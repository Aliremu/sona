"use client"

import { useState } from "react"
import { useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { ChevronDown, ChevronRight, Music, Plus, Play, Clock, Gauge, ChevronLeft, Menu, Trash2 } from "lucide-react"

interface Song {
  id: number
  title: string
  subtitle: string
  videoUrl: string
  sheetMusicUrl: string
  bpm: number
  duration: string
}

interface Playlist {
  id: number
  name: string
  songs: Song[]
}

interface PlaylistSidebarProps {
  playlists: Playlist[]
  currentSong: Song
  onSongSelect: (song: Song, artistName: string) => void
  onAddPlaylist: (name: string) => void
  onRemovePlaylist: (playlistId: number) => void
  onAddSong: (playlistId: number, song: Song) => void
  isCollapsed: boolean
  onToggleCollapse: () => void
  children?: React.ReactNode
}

export function PlaylistSidebar({
  playlists,
  currentSong,
  onSongSelect,
  onAddPlaylist,
  onRemovePlaylist,
  onAddSong,
  isCollapsed,
  onToggleCollapse,
  children,
}: PlaylistSidebarProps) {
  const [expandedPlaylists, setExpandedPlaylists] = useState<number[]>([1])

  // Set CSS custom property for sidebar width
  useEffect(() => {
    const sidebarWidth = isCollapsed ? '64px' : '320px'
    document.documentElement.style.setProperty('--sidebar-width', sidebarWidth)
  }, [isCollapsed])

  const [newPlaylistName, setNewPlaylistName] = useState("")
  const [newSongData, setNewSongData] = useState({
    title: "",
    subtitle: "",
    videoUrl: "",
    bpm: 120,
    duration: "0:00",
  })
  const [selectedPlaylistId, setSelectedPlaylistId] = useState<number | null>(null)

  const togglePlaylist = (playlistId: number) => {
    setExpandedPlaylists((prev) =>
      prev.includes(playlistId) ? prev.filter((id) => id !== playlistId) : [...prev, playlistId],
    )
  }

  const handleAddPlaylist = () => {
    if (newPlaylistName.trim()) {
      onAddPlaylist(newPlaylistName.trim())
      setNewPlaylistName("")
    }
  }

  const handleAddSong = () => {
    if (selectedPlaylistId && newSongData.title.trim()) {
      const newSong = {
        id: Date.now(),
        ...newSongData,
        sheetMusicUrl: "/guitar-sheet-music.png",
      }
      onAddSong(selectedPlaylistId, newSong)
      setNewSongData({
        title: "",
        subtitle: "",
        videoUrl: "",
        bpm: 120,
        duration: "0:00",
      })
      setSelectedPlaylistId(null)
    }
  }

  return (
    <div 
      className={`${isCollapsed ? "w-16" : ""} border-r bg-muted/30 flex flex-col transition-all duration-300 min-w-[320px] relative overflow-hidden`}
      style={{
        width: isCollapsed ? '64px' : '320px',
        minWidth: isCollapsed ? '64px' : '320px'
      }}
    >
      {/* Header */}
      <div className="p-4 border-b">
        <div className="flex items-center justify-between mb-3">
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 hover:bg-accent/20"
            onClick={onToggleCollapse}
          >
            {isCollapsed ? (
              <Menu className="h-4 w-4 text-foreground" />
            ) : (
              <ChevronLeft className="h-4 w-4 text-foreground" />
            )}
          </Button>
          {!isCollapsed && (
            <>
              <h2 className="font-semibold tracking-tight">Playlists</h2>
              <Dialog>
                <DialogTrigger asChild>
                  <Button variant="ghost" size="icon" className="h-8 w-8 hover:bg-accent/20">
                    <Plus className="h-4 w-4 text-foreground" />
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Create New Playlist</DialogTitle>
                    <DialogDescription>Add a new artist or playlist to organize your practice songs.</DialogDescription>
                  </DialogHeader>
                  <div className="space-y-4">
                    <div className="space-y-2">
                      <Label htmlFor="playlist-name">Playlist Name</Label>
                      <Input
                        id="playlist-name"
                        placeholder="e.g., ZUTOMAYO, Yorushika..."
                        value={newPlaylistName}
                        onChange={(e) => setNewPlaylistName(e.target.value)}
                        className="focus:ring-accent focus:border-accent"
                      />
                    </div>
                  </div>
                  <DialogFooter>
                    <Button onClick={handleAddPlaylist} className="bg-accent hover:bg-accent/90 text-accent-foreground">
                      Create Playlist
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            </>
          )}
        </div>
        {!isCollapsed && <p className="text-xs text-muted-foreground">Organize your practice songs by artist</p>}
      </div>

      {/* Playlists */}
      <div className="flex-1 pb-16 min-h-0">
      <ScrollArea className="flex-1 h-full">
        <div className="p-2 space-y-1">
          {playlists.map((playlist) => (
            <Collapsible
              key={playlist.id}
              open={!isCollapsed && expandedPlaylists.includes(playlist.id)}
              onOpenChange={() => !isCollapsed && togglePlaylist(playlist.id)}
            >
              <div className="flex items-center w-full">
                <CollapsibleTrigger asChild>
                  <Button
                    variant="ghost"
                    className={`flex-1 ${isCollapsed ? "justify-center px-2" : "justify-between px-3"} h-10 font-medium hover:bg-accent/20`}
                    onClick={() => isCollapsed && onToggleCollapse()}
                  >
                    <div className="flex items-center gap-2">
                      <Music className="h-4 w-4 text-accent" />
                      {!isCollapsed && (
                        <>
                          <span className="text-foreground">{playlist.name}</span>
                          <Badge variant="secondary" className="text-xs bg-accent/20 text-accent border-accent/30">
                            {playlist.songs.length}
                          </Badge>
                        </>
                      )}
                    </div>
                    {!isCollapsed &&
                      (expandedPlaylists.includes(playlist.id) ? (
                        <ChevronDown className="h-4 w-4 text-muted-foreground" />
                      ) : (
                        <ChevronRight className="h-4 w-4 text-muted-foreground" />
                      ))}
                  </Button>
                </CollapsibleTrigger>
                {!isCollapsed && (
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 ml-1 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                    onClick={(e) => {
                      e.stopPropagation();
                      onRemovePlaylist(playlist.id);
                    }}
                    title="Remove playlist"
                  >
                    <Trash2 className="h-3 w-3" />
                  </Button>
                )}
              </div>
              {!isCollapsed && (
                <CollapsibleContent className="space-y-1">
                  {playlist.songs.map((song) => (
                    <Button
                      key={song.id}
                      variant={currentSong.id === song.id ? "secondary" : "ghost"}
                      className={`w-full justify-start h-auto p-3 ml-6 ${
                        currentSong.id === song.id
                          ? "bg-accent/20 border-l-2 border-accent hover:bg-accent/30 text-foreground"
                          : "hover:bg-accent/10 text-foreground"
                      }`}
                      onClick={() => onSongSelect(song, playlist.name)}
                    >
                      <div className="flex items-start gap-3 w-full">
                        <div className="mt-0.5">
                          <Play
                            className={`h-3 w-3 ${currentSong.id === song.id ? "text-accent fill-accent" : "text-muted-foreground"}`}
                          />
                        </div>
                        <div className="flex-1 text-left space-y-1">
                          <p className="text-sm font-medium leading-none text-foreground">{song.title}</p>
                          <p className="text-xs text-muted-foreground">{song.subtitle}</p>
                          <div className="flex items-center gap-3 text-xs text-muted-foreground">
                            <div className="flex items-center gap-1">
                              <Clock className="h-3 w-3" />
                              {song.duration}
                            </div>
                            <div className="flex items-center gap-1">
                              <Gauge className="h-3 w-3" />
                              {song.bpm} BPM
                            </div>
                          </div>
                        </div>
                      </div>
                    </Button>
                  ))}
                  <Dialog>
                    <DialogTrigger asChild>
                      <Button
                        variant="ghost"
                        className="w-full justify-start h-8 ml-6 text-muted-foreground hover:text-accent hover:bg-accent/10"
                        onClick={() => setSelectedPlaylistId(playlist.id)}
                      >
                        <Plus className="h-3 w-3 mr-2" />
                        <span className="text-xs">Add song</span>
                      </Button>
                    </DialogTrigger>
                    <DialogContent>
                      <DialogHeader>
                        <DialogTitle>Add Song to {playlist.name}</DialogTitle>
                        <DialogDescription>Add a new practice song with video and sheet music.</DialogDescription>
                      </DialogHeader>
                      <div className="space-y-4">
                        <div className="grid grid-cols-2 gap-4">
                          <div className="space-y-2">
                            <Label htmlFor="song-title">Song Title</Label>
                            <Input
                              id="song-title"
                              placeholder="Song name"
                              value={newSongData.title}
                              onChange={(e) => setNewSongData({ ...newSongData, title: e.target.value })}
                              className="focus:ring-accent focus:border-accent"
                            />
                          </div>
                          <div className="space-y-2">
                            <Label htmlFor="song-subtitle">Subtitle</Label>
                            <Input
                              id="song-subtitle"
                              placeholder="e.g., Guitar Cover"
                              value={newSongData.subtitle}
                              onChange={(e) => setNewSongData({ ...newSongData, subtitle: e.target.value })}
                              className="focus:ring-accent focus:border-accent"
                            />
                          </div>
                        </div>
                        <div className="space-y-2">
                          <Label htmlFor="video-url">YouTube Video URL</Label>
                          <Input
                            id="video-url"
                            placeholder="https://www.youtube.com/watch?v=..."
                            value={newSongData.videoUrl}
                            onChange={(e) => setNewSongData({ ...newSongData, videoUrl: e.target.value })}
                            className="focus:ring-accent focus:border-accent"
                          />
                        </div>
                        <div className="grid grid-cols-2 gap-4">
                          <div className="space-y-2">
                            <Label htmlFor="song-bpm">BPM</Label>
                            <Input
                              id="song-bpm"
                              type="number"
                              placeholder="120"
                              value={newSongData.bpm}
                              onChange={(e) =>
                                setNewSongData({ ...newSongData, bpm: Number.parseInt(e.target.value) || 120 })
                              }
                              className="focus:ring-accent focus:border-accent"
                            />
                          </div>
                          <div className="space-y-2">
                            <Label htmlFor="song-duration">Duration</Label>
                            <Input
                              id="song-duration"
                              placeholder="3:45"
                              value={newSongData.duration}
                              onChange={(e) => setNewSongData({ ...newSongData, duration: e.target.value })}
                              className="focus:ring-accent focus:border-accent"
                            />
                          </div>
                        </div>
                      </div>
                      <DialogFooter>
                        <Button onClick={handleAddSong} className="bg-accent hover:bg-accent/90 text-accent-foreground">
                          Add Song
                        </Button>
                      </DialogFooter>
                    </DialogContent>
                  </Dialog>
                </CollapsibleContent>
              )}
            </Collapsible>
          ))}
        </div>
      </ScrollArea>
      </div>
      {children}
    </div>
  )
}
