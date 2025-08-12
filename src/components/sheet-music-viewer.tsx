"use client"

import { useEffect, useRef, useState } from "react"
import { Button } from "@/components/ui/button"
import { Slider } from "@/components/ui/slider"
import { ChevronLeft, ChevronRight, ZoomIn, ZoomOut } from "lucide-react"

interface SheetMusicViewerProps {
  sheetMusicUrl?: string
  currentPage?: number
  onPageChange?: (page: number) => void
}

export function SheetMusicViewer({
  sheetMusicUrl = "/guitar-sheet-music.png",
  currentPage = 1,
  onPageChange,
}: SheetMusicViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [zoom, setZoom] = useState(100)
  const [totalPages] = useState(5)
  const [page, setPage] = useState(currentPage)

  useEffect(() => {
    if (currentPage !== page) {
      setPage(currentPage)
    }
  }, [currentPage])

  const handlePageChange = (newPage: number) => {
    if (newPage >= 1 && newPage <= totalPages) {
      setPage(newPage)
      onPageChange?.(newPage)
    }
  }

  const handleZoomChange = (value: number[]) => {
    setZoom(value[0])
  }

  return (
    <div className="flex flex-col h-full">
      {/* Controls */}
      <div className="flex items-center justify-between p-4 border-b bg-muted/30">
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="icon"
            className="h-8 w-8 bg-transparent"
            onClick={() => handlePageChange(page - 1)}
            disabled={page <= 1}
          >
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <span className="text-sm font-medium px-2">
            {page} / {totalPages}
          </span>
          <Button
            variant="outline"
            size="icon"
            className="h-8 w-8 bg-transparent"
            onClick={() => handlePageChange(page + 1)}
            disabled={page >= totalPages}
          >
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>

        <div className="flex items-center gap-3">
          <ZoomOut className="h-4 w-4 text-muted-foreground" />
          <Slider value={[zoom]} min={50} max={200} step={10} onValueChange={handleZoomChange} className="w-24" />
          <ZoomIn className="h-4 w-4 text-muted-foreground" />
          <span className="text-sm font-medium w-12 text-center">{zoom}%</span>
        </div>
      </div>

      {/* Sheet Music Display */}
      <div ref={containerRef} className="flex-1 overflow-auto bg-white">
        <div
          style={{ transform: `scale(${zoom / 100})`, transformOrigin: "top center" }}
          className="p-8 transition-transform"
        >
          <img
            src={sheetMusicUrl || "/placeholder.svg"}
            alt="Sheet Music"
            className="max-w-none shadow-lg rounded-lg"
          />
        </div>
      </div>
    </div>
  )
}
