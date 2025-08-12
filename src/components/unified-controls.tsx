"use client"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Slider } from "@/components/ui/slider"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Separator } from "@/components/ui/separator"
import {
  FastForward,
  Pause,
  Play,
  Repeat,
  SkipBack,
  Volume2,
  RotateCcw,
  ChevronLeft,
  ChevronRight,
  Gauge,
} from "lucide-react"

interface Song {
  id: number
  title: string
  subtitle: string
  videoUrl: string
  sheetMusicUrl: string
  bpm: number
  duration: string
}

interface UnifiedControlsProps {
  isPlaying: boolean
  onPlayStateChange: (playing: boolean) => void
  currentTime: number
  duration: number
  onSeek: (time: number) => void
  volume: number
  onVolumeChange: (volume: number) => void
  bpm: number
  onBpmChange: (bpm: number) => void
  playbackRate: number
  onPlaybackRateChange: (rate: number) => void
  pitch: number
  onPitchChange: (pitch: number) => void
  loopStart: string
  onLoopStartChange: (start: string) => void
  loopEnd: string
  onLoopEndChange: (end: string) => void
  isLooping: boolean
  onLoopingChange: (looping: boolean) => void
  currentSong: Song
  showPageControls?: boolean
  currentPage?: number
  onPageChange?: (page: number) => void
}

export function UnifiedControls({
  isPlaying,
  onPlayStateChange,
  currentTime,
  duration,
  onSeek,
  volume,
  onVolumeChange,
  bpm,
  onBpmChange,
  playbackRate,
  onPlaybackRateChange,
  pitch,
  onPitchChange,
  loopStart,
  onLoopStartChange,
  loopEnd,
  onLoopEndChange,
  isLooping,
  onLoopingChange,
  currentSong,
  showPageControls = false,
  currentPage = 1,
  onPageChange,
}: UnifiedControlsProps) {
  const formatTime = (time: number) => {
    const minutes = Math.floor(time / 60)
    const seconds = Math.floor(time % 60)
    return `${minutes}:${seconds.toString().padStart(2, "0")}`
  }

  const handleSeek = (value: number[]) => {
    const newTime = (value[0] / 100) * duration
    onSeek(newTime)
  }

  const skipBackward = () => onSeek(Math.max(0, currentTime - 10))
  const skipForward = () => onSeek(Math.min(duration, currentTime + 10))

  const setLoop = () => {
    const startTime = Number.parseFloat(loopStart)
    const endTime = Number.parseFloat(loopEnd)
    if (!isNaN(startTime) && !isNaN(endTime) && endTime > startTime) {
      onLoopingChange(true)
    }
  }

  return (
    <div className="bg-muted/30 border-t">
      <div className="p-4 space-y-4">
        {/* Progress Bar */}
        <div className="space-y-2">
          <Slider
            value={[(currentTime / duration) * 100]}
            max={100}
            step={0.1}
            onValueChange={handleSeek}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>{formatTime(currentTime)}</span>
            <span>{currentSong.duration}</span>
          </div>
        </div>

        {/* Main Controls Row */}
        <div className="flex items-center justify-between">
          {/* Playback Controls */}
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="icon" onClick={skipBackward}>
              <SkipBack className="h-4 w-4" />
            </Button>
            <Button size="icon" onClick={() => onPlayStateChange(!isPlaying)} className="h-10 w-10">
              {isPlaying ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
            </Button>
            <Button variant="ghost" size="icon" onClick={skipForward}>
              <FastForward className="h-4 w-4" />
            </Button>
            <Button variant={isLooping ? "default" : "ghost"} size="icon" onClick={() => onLoopingChange(!isLooping)}>
              <Repeat className="h-4 w-4" />
            </Button>
          </div>

          {/* Center Controls */}
          <div className="flex items-center gap-6">
            {/* Speed Control */}
            <div className="flex items-center gap-2">
              <Label className="text-sm font-medium">Speed</Label>
              <Input
                type="number"
                min="0.1"
                max="2"
                step="0.1"
                defaultValue={playbackRate}
                key={playbackRate} // Force re-render when playbackRate changes externally
                onBlur={(e) => {
                  const value = parseFloat(e.target.value);
                  if (!isNaN(value) && value >= 0.1 && value <= 2) {
                    onPlaybackRateChange(value);
                  } else {
                    // Reset to current value if invalid
                    e.target.value = playbackRate.toString();
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    const value = parseFloat(e.currentTarget.value);
                    if (!isNaN(value) && value >= 0.1 && value <= 2) {
                      onPlaybackRateChange(value);
                    } else {
                      // Reset to current value if invalid
                      e.currentTarget.value = playbackRate.toString();
                    }
                    e.currentTarget.blur();
                  }
                }}
                className="w-16 h-8 text-center"
              />
              <span className="text-xs text-muted-foreground">×</span>
            </div>

            {/* Pitch Control */}
            <div className="flex items-center gap-2">
              <Label className="text-sm font-medium">Pitch</Label>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 px-2 text-xs bg-transparent"
                  onClick={() => onPitchChange(Math.max(-12, pitch - 1))}
                >
                  -
                </Button>
                <Badge variant="outline" className="text-xs w-12 justify-center">
                  {pitch > 0 ? `+${pitch}` : pitch}
                </Badge>
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 px-2 text-xs bg-transparent"
                  onClick={() => onPitchChange(Math.min(12, pitch + 1))}
                >
                  +
                </Button>
              </div>
            </div>

            {/* BPM Control */}
            <div className="flex items-center gap-2">
              <Gauge className="h-4 w-4" />
              <Label htmlFor="bpm" className="text-sm font-medium">
                BPM
              </Label>
              <Input
                id="bpm"
                type="number"
                value={bpm}
                onChange={(e) => onBpmChange(Number.parseInt(e.target.value) || 120)}
                className="w-16 h-8 text-center"
              />
            </div>

            {/* Page Controls (Sheet Music only) */}
            {showPageControls && (
              <>
                <Separator orientation="vertical" className="h-6" />
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="icon"
                    className="h-8 w-8 bg-transparent"
                    onClick={() => onPageChange?.(Math.max(1, currentPage - 1))}
                    disabled={currentPage <= 1}
                  >
                    <ChevronLeft className="h-4 w-4" />
                  </Button>
                  <span className="text-sm font-medium px-2">Page {currentPage}</span>
                  <Button
                    variant="outline"
                    size="icon"
                    className="h-8 w-8 bg-transparent"
                    onClick={() => onPageChange?.(currentPage + 1)}
                    disabled={currentPage >= 5}
                  >
                    <ChevronRight className="h-4 w-4" />
                  </Button>
                </div>
              </>
            )}
          </div>

          {/* Volume Control */}
          <div className="flex items-center gap-2">
            <Volume2 className="h-4 w-4" />
            <Slider
              value={[volume]}
              max={100}
              step={1}
              className="w-20"
              onValueChange={(value) => onVolumeChange(value[0])}
            />
            <span className="text-xs text-muted-foreground w-8">{volume}%</span>
          </div>
        </div>

        {/* Loop Controls */}
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <RotateCcw className="h-4 w-4" />
            <Label className="text-sm font-medium">Loop Section</Label>
          </div>
          <div className="flex items-center gap-2">
            <Input
              placeholder="Start (s)"
              className="w-20 h-8 text-center"
              value={loopStart}
              onChange={(e) => onLoopStartChange(e.target.value)}
            />
            <span className="text-sm text-muted-foreground">to</span>
            <Input
              placeholder="End (s)"
              className="w-20 h-8 text-center"
              value={loopEnd}
              onChange={(e) => onLoopEndChange(e.target.value)}
            />
            <Button variant="outline" size="sm" onClick={setLoop}>
              Set Loop
            </Button>
            {isLooping && (
              <Button variant="outline" size="sm" onClick={() => onLoopingChange(false)}>
                Clear
              </Button>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
