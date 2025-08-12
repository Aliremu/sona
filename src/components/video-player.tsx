"use client"
import { useRef, forwardRef, useImperativeHandle } from "react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent } from "@/components/ui/card"
import { ExternalLink, Gauge } from "lucide-react"

interface Song {
  id: number
  title: string
  subtitle: string
  videoUrl: string
  sheetMusicUrl: string
  bpm: number
  duration: string
}

interface VideoPlayerProps {
  currentSong: Song
  isPlaying: boolean
  currentTime: number
  onTimeUpdate: (time: number) => void
}

export const VideoPlayer = forwardRef<HTMLVideoElement, VideoPlayerProps>(
  ({ currentSong, isPlaying, currentTime: _currentTime, onTimeUpdate: _onTimeUpdate }, ref) => {
    const iframeRef = useRef<HTMLIFrameElement>(null)
    const dummyVideoRef = useRef<HTMLVideoElement>(null)
    useImperativeHandle(ref, () => dummyVideoRef.current!, [])

    const openInYouTube = () => {
      const videoId = currentSong.videoUrl.split("/").pop()?.split("?")[0]
      if (videoId) {
        window.open(`https://www.youtube.com/watch?v=${videoId}`, "_blank")
      }
    }

    return (
      <div className="relative bg-black h-full">
        <iframe
          ref={iframeRef}
          className="w-full h-full"
          src={currentSong.videoUrl}
          title={`${currentSong.title} - ${currentSong.subtitle}`}
          allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
          allowFullScreen
        />

        <video ref={dummyVideoRef} className="hidden" crossOrigin="anonymous" muted>
          <source src="/sample-guitar-video.mp4" type="video/mp4" />
        </video>

        {/* Video Info Overlay */}
        <div className="absolute top-4 left-4">
          <Card className="bg-black/80 border-white/20">
            <CardContent className="p-3">
              <div className="flex items-center gap-3">
                <div className="text-white">
                  <p className="text-sm font-medium">{currentSong.title}</p>
                  <p className="text-xs text-white/70">{currentSong.subtitle}</p>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6 text-white hover:bg-white/20"
                  onClick={openInYouTube}
                >
                  <ExternalLink className="h-3 w-3" />
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* BPM Overlay */}
        <div className="absolute top-4 right-4">
          <Card className="bg-black/80 border-white/20">
            <CardContent className="p-3">
              <div className="flex items-center gap-2 text-white">
                <Gauge className="h-3 w-3" />
                <span className="text-xs">BPM: {currentSong.bpm}</span>
                {isPlaying && (
                  <Badge variant="secondary" className="text-xs">
                    PLAYING
                  </Badge>
                )}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    )
  },
)

VideoPlayer.displayName = "VideoPlayer"
