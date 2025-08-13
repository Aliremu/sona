"use client"

import { useEffect, useRef, useCallback } from "react"

interface AudioVisualizerProps {
  videoElement?: HTMLVideoElement | null
  isPlaying: boolean
}

export function AudioVisualizer({ videoElement, isPlaying }: AudioVisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const animationRef = useRef<number | null>(null)
  const audioContextRef = useRef<AudioContext | null>(null)
  const analyserRef = useRef<AnalyserNode | null>(null)
  const sourceRef = useRef<MediaElementAudioSourceNode | null>(null)

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

    let frequencyData: Uint8Array | null = null

    // Get real audio data if available
    if (analyserRef.current) {
      const bufferLength = analyserRef.current.frequencyBinCount
      const dataArray = new Uint8Array(new ArrayBuffer(bufferLength))
      analyserRef.current.getByteFrequencyData(dataArray)
      frequencyData = dataArray
    }

    // Generate frequency bars
    const barCount = 64
    const barWidth = (width / barCount) * 0.8
    const gap = (width / barCount) * 0.2

    for (let i = 0; i < barCount; i++) {
      let frequency: number

      if (frequencyData) {
        // Use real audio data
        const dataIndex = Math.floor((i / barCount) * (frequencyData.length / 2))
        frequency = frequencyData[dataIndex] / 255
      } else {
        // Fallback to animated bars
        frequency = Math.sin(Date.now() * 0.002 + i * 0.15) * 0.5 + 0.5
      }

      const barHeight = frequency * height * 0.7

      // Color based on frequency range
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

  // Set up audio context and analyser when video element is available
  useEffect(() => {
    if (!videoElement) return

    const setupAudioContext = async () => {
      try {
        // Clean up existing audio context
        if (audioContextRef.current) {
          await audioContextRef.current.close()
        }

        // Create new audio context
        const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
        audioContextRef.current = audioContext

        // Create analyser
        const analyser = audioContext.createAnalyser()
        analyser.fftSize = 256
        analyser.smoothingTimeConstant = 0.8
        analyserRef.current = analyser

        // Create source from video element (only if not already created)
        if (!sourceRef.current) {
          const source = audioContext.createMediaElementSource(videoElement)
          sourceRef.current = source
          
          // Connect source to analyser and destination
          source.connect(analyser)
          source.connect(audioContext.destination)
        }
      } catch (error) {
        console.warn('Failed to setup audio context:', error)
      }
    }

    setupAudioContext()

    return () => {
      // Cleanup on unmount
      if (audioContextRef.current && audioContextRef.current.state !== 'closed') {
        audioContextRef.current.close()
      }
    }
  }, [videoElement])

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
