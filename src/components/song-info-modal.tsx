"use client"

import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Badge } from "@/components/ui/badge"
import { Separator } from "@/components/ui/separator"
import { Music, Clock, Gauge, User, Tag } from "lucide-react"

interface Song {
  id: number
  title: string
  subtitle: string
  videoUrl: string
  sheetMusicUrl: string
  bpm: number
  duration: string
}

interface SongInfoModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentSong: Song
  currentArtist: string
  bpm: number
}

export function SongInfoModal({ open, onOpenChange, currentSong, currentArtist, bpm }: SongInfoModalProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Music className="h-5 w-5" />
            Song Information
          </DialogTitle>
          <DialogDescription>Details about the current song</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <div className="h-12 w-12 rounded-lg bg-primary/10 flex items-center justify-center">
              <Music className="h-6 w-6 text-primary" />
            </div>
            <div className="flex-1">
              <h3 className="font-semibold">{currentSong.title}</h3>
              <p className="text-sm text-muted-foreground">{currentSong.subtitle}</p>
            </div>
          </div>

          <Separator />

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm">
                <User className="h-4 w-4 text-muted-foreground" />
                <span className="text-muted-foreground">Artist</span>
              </div>
              <Badge variant="outline">{currentArtist}</Badge>
            </div>

            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm">
                <Tag className="h-4 w-4 text-muted-foreground" />
                <span className="text-muted-foreground">Type</span>
              </div>
              <Badge variant="secondary">{currentSong.subtitle}</Badge>
            </div>

            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm">
                <Clock className="h-4 w-4 text-muted-foreground" />
                <span className="text-muted-foreground">Duration</span>
              </div>
              <p className="font-medium">{currentSong.duration}</p>
            </div>

            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm">
                <Gauge className="h-4 w-4 text-muted-foreground" />
                <span className="text-muted-foreground">BPM</span>
              </div>
              <Badge variant="outline">{bpm}</Badge>
            </div>
          </div>

          <Separator />

          <div className="space-y-2">
            <h4 className="text-sm font-medium">Resources</h4>
            <div className="space-y-1 text-sm text-muted-foreground">
              <p>• Video tutorial available</p>
              <p>• Sheet music with TAB notation</p>
              <p>• Practice backing track</p>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
