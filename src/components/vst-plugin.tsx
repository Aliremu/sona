"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Switch } from "@/components/ui/switch"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader } from "@/components/ui/card"
import { Settings, Trash2, Sliders } from "lucide-react"

interface VstPluginProps {
  id: number
  name: string
  type: string
  enabled: boolean
  onToggle: (id: number) => void
  onRemove: (id: number) => void
}

export function VstPlugin({ id, name, type, enabled, onToggle, onRemove }: VstPluginProps) {
  const [pluginName, setPluginName] = useState(name)
  const [isEditing, setIsEditing] = useState(false)
  const [parameters, setParameters] = useState({
    gain: 75,
    mix: 100,
    attack: 50,
    release: 60,
  })

  const handleParameterChange = (param: keyof typeof parameters, value: number[]) => {
    setParameters((prev) => ({
      ...prev,
      [param]: value[0],
    }))
  }

  return (
    <Card className={`transition-all ${enabled ? "border-primary/50 bg-card" : "border-border bg-muted/50"}`}>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Switch id={`plugin-${id}`} checked={enabled} onCheckedChange={() => onToggle(id)} />
            <div className="flex items-center gap-2">
              <Sliders className="h-4 w-4 text-muted-foreground" />
              {isEditing ? (
                <Input
                  value={pluginName}
                  onChange={(e) => setPluginName(e.target.value)}
                  className="h-7 w-32 text-sm"
                  onBlur={() => setIsEditing(false)}
                  onKeyDown={(e) => e.key === "Enter" && setIsEditing(false)}
                  autoFocus
                />
              ) : (
                <Label
                  htmlFor={`plugin-${id}`}
                  className="font-medium cursor-pointer hover:text-primary"
                  onClick={() => setIsEditing(true)}
                >
                  {pluginName}
                </Label>
              )}
              <Badge variant="secondary" className="text-xs">
                {type}
              </Badge>
            </div>
          </div>
          <div className="flex gap-1">
            <Button variant="ghost" size="icon" className="h-8 w-8">
              <Settings className="h-3 w-3" />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => onRemove(id)}>
              <Trash2 className="h-3 w-3" />
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="pt-0">
        <div className="grid grid-cols-2 gap-4">
          {Object.entries(parameters).map(([param, value]) => (
            <div key={param} className="space-y-2">
              <div className="flex justify-between items-center">
                <Label className="text-xs font-medium capitalize">{param}</Label>
                <span className="text-xs text-muted-foreground">
                  {value}
                  {param === "attack" || param === "release" ? "ms" : "%"}
                </span>
              </div>
              <Slider
                value={[value]}
                max={100}
                step={1}
                onValueChange={(val) => handleParameterChange(param as keyof typeof parameters, val)}
                className="h-1"
              />
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
