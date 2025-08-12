"use client"

import { useEffect, useRef, useCallback } from "react"

interface AudioVisualizerProps {
  isPlaying: boolean
}

export function AudioVisualizer({ isPlaying }: AudioVisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const animationRef = useRef<number | null>(null)

  const renderFrame = useCallback(() => {
    if (!canvasRef.current) return

    const canvas = canvasRef.current
    const ctx = canvas.getContext("2d")
    if (!ctx) return

    const width = canvas.width
    const height = canvas.height

    // Clear with card background
    ctx.fillStyle = "hsl(var(--card))"
    ctx.fillRect(0, 0, width, height)

    if (!isPlaying) return

    // Generate animated frequency bars
    const barCount = 64
    const barWidth = (width / barCount) * 0.8
    const gap = (width / barCount) * 0.2

    for (let i = 0; i < barCount; i++) {
      const frequency = Math.sin(Date.now() * 0.002 + i * 0.15) * 0.5 + 0.5
      const barHeight = frequency * height * 0.7

      // Simple color based on frequency range
      let color = "hsl(var(--primary))"
      if (i < barCount / 3) {
        color = "#10b981" // emerald
      } else if (i < (barCount * 2) / 3) {
        color = "#3b82f6" // blue
      } else {
        color = "#8b5cf6" // purple
      }

      ctx.fillStyle = color
      ctx.fillRect(i * (barWidth + gap), height - barHeight, barWidth, barHeight)
    }

    // Draw center reference line
    ctx.strokeStyle = "hsl(var(--border))"
    ctx.lineWidth = 1
    ctx.setLineDash([5, 5])
    ctx.beginPath()
    ctx.moveTo(0, height / 2)
    ctx.lineTo(width, height / 2)
    ctx.stroke()
    ctx.setLineDash([])

    if (isPlaying) {
      animationRef.current = requestAnimationFrame(renderFrame)
    }
  }, [isPlaying])

  useEffect(() => {
    if (isPlaying) {
      renderFrame()
    } else {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current)
        animationRef.current = null
      }
      // Clear canvas when not playing
      if (canvasRef.current) {
        const ctx = canvasRef.current.getContext("2d")
        if (ctx) {
          ctx.fillStyle = "hsl(var(--card))"
          ctx.fillRect(0, 0, canvasRef.current.width, canvasRef.current.height)
        }
      }
    }

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current)
        animationRef.current = null
      }
    }
  }, [isPlaying, renderFrame])

  return (
    <div className="w-full h-full flex items-center justify-center">
      <canvas ref={canvasRef} className="w-full h-full rounded-md" width={800} height={400} />
      {!isPlaying && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center text-muted-foreground">
            <div className="h-12 w-12 rounded-full border-2 border-muted-foreground/20 flex items-center justify-center mx-auto mb-3">
              <div className="h-6 w-6 rounded-full bg-muted-foreground/20" />
            </div>
            <p className="text-sm">Play video to see visualization</p>
          </div>
        </div>
      )}
    </div>
  )
}
