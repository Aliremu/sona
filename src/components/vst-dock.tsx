import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Switch } from "@/components/ui/switch"
import { Slider } from "@/components/ui/slider"
import { Label } from "@/components/ui/label"
import { ChevronUp, ChevronDown, Plus, Sliders, Trash2 } from "lucide-react"

interface Plugin {
  id: number
  name: string
  enabled: boolean
  type: string
  color: string
}

interface VstDockProps {
  plugins: Plugin[]
  expanded: boolean
  sidebarCollapsed: boolean
  onToggleExpanded: () => void
  onAddPlugin: () => void
  onTogglePlugin: (id: number) => void
  onRemovePlugin: (id: number) => void
  onExpandSidebar?: () => void
}

export function VstDock({
  plugins,
  expanded,
  sidebarCollapsed,
  onToggleExpanded,
  onAddPlugin,
  onTogglePlugin,
  onRemovePlugin,
  onExpandSidebar,
}: VstDockProps) {
  // If sidebar is collapsed, show just a clickable icon
  if (sidebarCollapsed) {
    return (
      <div
        className="absolute bottom-0 left-0 w-16 h-16 bg-background/95 backdrop-blur-xl border-t border-r shadow-lg flex items-center justify-center cursor-pointer hover:bg-accent/10 transition-colors"
        onClick={() => {
          onExpandSidebar?.()
          onToggleExpanded() // Also expand the VST rack when clicked
        }}
      >
        <div className="h-6 w-6 rounded bg-primary flex items-center justify-center">
          <Sliders className="h-3 w-3 text-primary-foreground" />
        </div>
      </div>
    )
  }
  return (
    <div
      className={`absolute bottom-0 left-0 bg-background/95 backdrop-blur-xl border-t shadow-lg transition-all duration-300 ease-in-out ${
        expanded ? "h-72" : "h-[70px]"
      } w-full`}
      style={{
        // Inherit width from parent (playlist sidebar). Min width enforced there.
        bottom: 0,
      }}
    >
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div className="flex items-center gap-3">
          <div className="h-6 w-6 rounded bg-primary flex items-center justify-center">
            <Sliders className="h-3 w-3 text-primary-foreground" />
          </div>
          <div>
            <h3 className="font-medium text-sm">VST Plugin Rack</h3>
            <p className="text-xs text-muted-foreground">
              {plugins.filter((p) => p.enabled).length} of {plugins.length} active
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button onClick={onAddPlugin} size="sm" className="h-7 px-3">
            <Plus className="h-3 w-3 mr-1" />
            Add
          </Button>
          <Button variant="ghost" size="icon" onClick={onToggleExpanded} className="h-8 w-8">
            {expanded ? <ChevronDown className="h-4 w-4" /> : <ChevronUp className="h-4 w-4" />}
          </Button>
        </div>
      </div>

      {expanded && (
        <ScrollArea className="h-[calc(100%-64px)]">
          <div className="p-2 space-y-1">
            {plugins.map((plugin) => (
              <div
                key={plugin.id}
                className={`flex items-center gap-3 p-2 rounded-lg border transition-all duration-200 ${
                  plugin.enabled ? "bg-card border-border shadow-sm" : "bg-muted/50 border-border/50"
                }`}
              >
                <Switch
                  id={`dock-plugin-${plugin.id}`}
                  checked={plugin.enabled}
                  onCheckedChange={() => onTogglePlugin(plugin.id)}
                  className="scale-75"
                />
                <div className="flex-1">
                  <Label htmlFor={`dock-plugin-${plugin.id}`} className="text-sm font-medium">
                    {plugin.name}
                  </Label>
                </div>
                <div className="w-20">
                  <Slider defaultValue={[75]} max={100} step={1} className="h-1" />
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6 text-destructive hover:text-destructive"
                  onClick={() => onRemovePlugin(plugin.id)}
                >
                  <Trash2 className="h-3 w-3" />
                </Button>
              </div>
            ))}
          </div>
        </ScrollArea>
      )}
    </div>
  )
}
