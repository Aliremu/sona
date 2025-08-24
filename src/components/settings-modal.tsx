import { useEffect, useState } from "react"
import { useTheme } from "next-themes"
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Separator } from "@/components/ui/separator"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import { Slider } from "@/components/ui/slider"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Settings, Folder, Volume2, Mic, Headphones, Sliders, Palette, Plus, Trash2 } from "lucide-react"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { toast } from "sonner"
import { APP_NAME, APP_VERSION, APP_DESCRIPTION, APP_AUTHOR } from "@/lib/constants"

interface SettingsModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SettingsModal({ open, onOpenChange }: SettingsModalProps) {
  const { theme, setTheme } = useTheme()
  const [activeTab, setActiveTab] = useState("audio")
  const [pluginDirectories, setPluginDirectories] = useState<string[]>([])
  const [newPluginDirectory, setNewPluginDirectory] = useState("")
  const [discoveredPlugins, setDiscoveredPlugins] = useState<string[]>([])
  const [cpuUsage, setCpuUsage] = useState<number>(0)

  const [availableHosts, setAvailableHosts] = useState<string[]>([])
  const [audioDriver, setAudioDriver] = useState("")

  const [availableInputDevices, setAvailableInputDevices] = useState<string[]>([])
  const [inputDevice, setInputDevice] = useState("")

  const [availableOutputDevices, setAvailableOutputDevices] = useState<string[]>([])
  const [outputDevice, setOutputDevice] = useState("")

  const [sampleRate, setSampleRate] = useState("44100")
  const [bufferSize, setBufferSize] = useState("256")
  const [inputGain, setInputGain] = useState(75)
  const [outputGain, setOutputGain] = useState(80)
  const [enableLowLatency, setEnableLowLatency] = useState(true)
  const [enableMetronome, setEnableMetronome] = useState(false)

  // Load plugin paths on component mount
  useEffect(() => {
    async function loadPluginData() {
      try {
        const paths: string[] = await invoke('get_plugin_paths')
        setPluginDirectories(paths)
        
        const plugins: string[] = await invoke('get_discovered_plugins')
        setDiscoveredPlugins(plugins)
      } catch (error) {
        console.error('Failed to load plugin data:', error)
      }
    }
    
    if (open) {
      loadPluginData()
    }
  }, [open])

  // Set up event listeners for directory selection
  useEffect(() => {
    if (!open) return

    let unlistenSelected: (() => void) | undefined
    let unlistenCancelled: (() => void) | undefined

    async function setupListeners() {
      unlistenSelected = await listen<string>('directory-selected', (event) => {
        setNewPluginDirectory(event.payload)
      })

      unlistenCancelled = await listen('directory-cancelled', () => {
        // User cancelled, no action needed
        console.log('Directory selection cancelled')
      })
    }

    setupListeners()

    return () => {
      unlistenSelected?.()
      unlistenCancelled?.()
    }
  }, [open])

  // Update CPU usage periodically when plugins tab is active
  useEffect(() => {
    if (!open || activeTab !== "plugins") return

    const updateCpuUsage = async () => {
      try {
        const usage: number = await invoke('get_cpu_usage')
        setCpuUsage(Math.round(usage))
      } catch (error) {
        console.error('Failed to get CPU usage:', error)
      }
    }

    // Update immediately
    updateCpuUsage()

    // Then update every 2 seconds
    const interval = setInterval(updateCpuUsage, 2000)

    return () => clearInterval(interval)
  }, [open, activeTab])

  // Save plugin paths when they change
  const savePluginPaths = async (paths: string[]) => {
    try {
      await invoke('set_plugin_paths', { paths })
      console.log('Plugin paths saved:', paths)
      toast.success('Plugin directories updated successfully')
      return true
    } catch (error) {
      console.error('Failed to save plugin paths:', error)
      toast.error(typeof error === 'string' ? error : 'Failed to update plugin directories')
      return false
    }
  }

  // Add a new plugin directory
  const addPluginDirectory = async () => {
    const trimmedPath = newPluginDirectory.trim()
    if (!trimmedPath) {
      toast.error('Please enter a directory path')
      return
    }
    
    if (pluginDirectories.includes(trimmedPath)) {
      toast.error('Directory already exists in the list')
      return
    }
    
    const updatedPaths = [...pluginDirectories, trimmedPath]
    
    // Try to save to backend first
    const success = await savePluginPaths(updatedPaths)
    
    // Only update frontend state if backend save was successful
    if (success) {
      setPluginDirectories(updatedPaths)
      setNewPluginDirectory("")
    }
  }

  // Remove a plugin directory
  const removePluginDirectory = async (index: number) => {
    const updatedPaths = pluginDirectories.filter((_, i) => i !== index)
    
    // Try to save to backend first
    const success = await savePluginPaths(updatedPaths)
    
    // Only update frontend state if backend save was successful
    if (success) {
      setPluginDirectories(updatedPaths)
    }
  }

  const scanPlugins = () => {
    invoke('scan_plugins')
      .then((plugins) => {
        setDiscoveredPlugins(plugins as string[])
      })
      .catch((error) => {
        console.error('Failed to scan plugins:', error)
        toast.error('Failed to scan plugins')
      })
  }

  // Browse for directory
  const browseDirectory = async () => {
    try {
      await invoke('browse_directory')
      // The result will come through the event listener
    } catch (error) {
      console.error('Failed to browse directory:', error)
      toast.error('Failed to open directory browser')
    }
  }

  useEffect(() => {
    async function run() {
      const response = await invoke('select_host', { host: audioDriver });

      const inputDevices: string[] = await invoke('get_input_devices');
      setAvailableInputDevices(inputDevices);

      const outputDevices: string[] = await invoke('get_output_devices');
      setAvailableOutputDevices(outputDevices);

      setInputDevice(await invoke('get_input_device'));
      setOutputDevice(await invoke('get_output_device'));

      const size = (await invoke('get_buffer_size') as number).toString();
      console.log("Fetched buffer size:", size);
      setBufferSize(size);

      console.log("Selected audio driver:", response);
      console.log("Set input device:", inputDevice);
      console.log("Set output device:", outputDevice);
    }

    run();
  }, [audioDriver]);

  useEffect(() => {
    async function run() {
      const response = await invoke('select_input', { inputDevice: inputDevice });
      console.log("Selected input device:", response);
    }

    run();
  }, [inputDevice]);

  useEffect(() => {
    async function run() {
      const response = await invoke('select_output', { outputDevice: outputDevice });
      console.log("Selected output device:", response);
    }

    run();
  }, [outputDevice]);

  useEffect(() => {
    async function run() {
      const hosts = await invoke('get_hosts');
      setAvailableHosts(hosts as string[]);
      setAudioDriver(await invoke('get_host') as string);
    }

    run();
  }, []);

  useEffect(() => {
    async function run() {
      const response = await invoke('set_buffer_size', { size: parseInt(bufferSize) });
      console.log("Set buffer size:", response);
    }

    run();
  }, [bufferSize]);

  const tabs = [
    { id: "audio", label: "Audio", icon: Volume2 },
    { id: "plugins", label: "Plugins", icon: Sliders },
    { id: "general", label: "General", icon: Palette },
  ]

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-4xl h-[85vh] p-0 flex flex-col">
        <DialogHeader className="p-6 pb-0 flex-shrink-0">
          <DialogTitle className="flex items-center gap-2">
            <div className="h-6 w-6 rounded bg-accent flex items-center justify-center">
              <Settings className="h-4 w-4 text-accent-foreground" />
            </div>
            Settings
          </DialogTitle>
          <DialogDescription>Configure your audio settings and plugin directories</DialogDescription>
        </DialogHeader>

        <div className="flex flex-1 overflow-hidden">
          {/* Side Panel */}
          <div className="w-48 border-r bg-muted/30 flex-shrink-0">
            <div className="p-4">
              <div className="space-y-1">
                {tabs.map((tab) => {
                  const Icon = tab.icon
                  return (
                    <Button
                      key={tab.id}
                      variant={activeTab === tab.id ? "secondary" : "ghost"}
                      className={`w-full justify-start h-10 ${
                        activeTab === tab.id
                          ? "bg-accent text-accent-foreground hover:bg-accent/90"
                          : "hover:bg-muted hover:text-foreground"
                      }`}
                      onClick={() => setActiveTab(tab.id)}
                    >
                      <Icon className="h-4 w-4 mr-2" />
                      {tab.label}
                    </Button>
                  )
                })}
              </div>
            </div>
          </div>

          {/* Content Area */}
          <div className="flex-1 flex flex-col overflow-hidden">
            <ScrollArea className="flex-1 overflow-auto">
              <div className="p-6 pb-24">
                {activeTab === "audio" && (
                  <div className="space-y-6">
                    {/* Audio Driver */}
                    <div className="space-y-3">
                      <Label className="text-sm font-medium flex items-center gap-2">
                        <div className="h-2 w-2 rounded-full bg-accent" />
                        Audio Driver
                      </Label>
                      <Select value={audioDriver} onValueChange={setAudioDriver}>
                        <SelectTrigger className="focus:ring-accent focus:border-accent w-24 max-w-24">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            {availableHosts.map((host) => (
                              <SelectItem key={host} value={host}>
                                {host}
                              </SelectItem>
                            ))}
                          </SelectContent>
                      </Select>
                    </div>

                    <Separator />

                    {/* Input/Output Devices */}
                    <div className="grid grid-cols-2 gap-6">
                      <div className="space-y-3">
                        <Label className="text-sm font-medium flex items-center gap-2">
                          <Mic className="h-4 w-4 text-accent" />
                          Input Device
                        </Label>
                        <Select value={inputDevice} onValueChange={setInputDevice}>
                          <SelectTrigger className="focus:ring-accent focus:border-accent w-64 max-w-64">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {availableInputDevices.map((device) => (
                              <SelectItem key={device} value={device}>
                                {device}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>

                      <div className="space-y-3">
                        <Label className="text-sm font-medium flex items-center gap-2">
                          <Headphones className="h-4 w-4 text-accent" />
                          Output Device
                        </Label>
                        <Select value={outputDevice} onValueChange={setOutputDevice}>
                          <SelectTrigger className="focus:ring-accent focus:border-accent w-64 max-w-64">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {availableOutputDevices.map((device) => (
                              <SelectItem key={device} value={device}>
                                {device}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                    </div>

                    <Separator />

                    {/* Audio Settings */}
                    <div className="grid grid-cols-2 gap-6">
                      <div className="space-y-3">
                        <Label className="text-sm font-medium">Sample Rate</Label>
                        <Select value={sampleRate} onValueChange={setSampleRate}>
                          <SelectTrigger className="focus:ring-accent focus:border-accent">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="44100">44.1 kHz</SelectItem>
                            <SelectItem value="48000">48 kHz</SelectItem>
                            <SelectItem value="88200">88.2 kHz</SelectItem>
                            <SelectItem value="96000">96 kHz</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      <div className="space-y-3">
                        <Label className="text-sm font-medium">Buffer Size</Label>
                        <Select value={bufferSize} onValueChange={setBufferSize}>
                          <SelectTrigger className="focus:ring-accent focus:border-accent">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="64">64 samples</SelectItem>
                            <SelectItem value="128">128 samples</SelectItem>
                            <SelectItem value="192">192 samples</SelectItem>
                            <SelectItem value="256">256 samples</SelectItem>
                            <SelectItem value="512">512 samples</SelectItem>
                            <SelectItem value="1024">1024 samples</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>

                    <Separator />

                    {/* Gain Controls */}
                    <div className="space-y-6">
                      <div className="space-y-3">
                        <Label className="text-sm font-medium">Input Gain</Label>
                        <div className="flex items-center gap-3">
                          <Slider
                            value={[inputGain]}
                            max={100}
                            step={1}
                            className="flex-1 [&_[role=slider]]:bg-accent [&_[role=slider]]:border-accent"
                            onValueChange={(value) => setInputGain(value[0])}
                          />
                          <div className="w-12 text-sm font-medium text-accent">{inputGain}%</div>
                        </div>
                      </div>

                      <div className="space-y-3">
                        <Label className="text-sm font-medium">Output Gain</Label>
                        <div className="flex items-center gap-3">
                          <Slider
                            value={[outputGain]}
                            max={100}
                            step={1}
                            className="flex-1 [&_[role=slider]]:bg-accent [&_[role=slider]]:border-accent"
                            onValueChange={(value) => setOutputGain(value[0])}
                          />
                          <div className="w-12 text-sm font-medium text-accent">{outputGain}%</div>
                        </div>
                      </div>
                    </div>

                    <Separator />

                    {/* Audio Options */}
                    <div className="space-y-4">
                      <div className="flex items-center justify-between p-3 rounded-lg border bg-card">
                        <div>
                          <Label htmlFor="low-latency" className="text-sm font-medium">
                            Enable Low Latency Mode
                          </Label>
                          <p className="text-xs text-muted-foreground mt-1">
                            Reduces audio latency but may increase CPU usage
                          </p>
                        </div>
                        <Switch
                          id="low-latency"
                          checked={enableLowLatency}
                          onCheckedChange={setEnableLowLatency}
                          className="data-[state=checked]:bg-accent"
                        />
                      </div>
                    </div>
                  </div>
                )}

                {activeTab === "plugins" && (
                  <div className="space-y-6">
                    {/* Plugin Directories */}
                    <div className="space-y-3">
                      <Label className="text-sm font-medium flex items-center gap-2">
                        <Folder className="h-4 w-4 text-accent" />
                        VST Plugin Directories
                      </Label>
                      
                      {/* Add new directory */}
                      <div className="flex gap-2">
                        <Input
                          value={newPluginDirectory}
                          onChange={(e) => setNewPluginDirectory(e.target.value)}
                          placeholder="/path/to/vst/plugins"
                          className="focus:ring-accent focus:border-accent"
                          onKeyDown={(e) => {
                            if (e.key === 'Enter') {
                              addPluginDirectory()
                            }
                          }}
                        />
                        <Button
                          variant="outline"
                          onClick={addPluginDirectory}
                          disabled={!newPluginDirectory.trim() || pluginDirectories.includes(newPluginDirectory.trim())}
                          className="hover:bg-muted hover:text-foreground border-border bg-transparent flex-shrink-0"
                        >
                          <Plus className="h-4 w-4 mr-1" />
                          Add
                        </Button>
                        <Button
                          variant="outline"
                          onClick={browseDirectory}
                          className="hover:bg-muted hover:text-foreground border-border bg-transparent flex-shrink-0"
                        >
                          Browse
                        </Button>
                      </div>

                      {/* Current directories list in scrollable viewport */}
                      <div className="border rounded-lg bg-card">
                        <div className="p-3 border-b bg-muted/30">
                          <div className="text-sm font-medium">Configured Directories ({pluginDirectories.length})</div>
                          <div className="text-xs text-muted-foreground">
                            Add directories where your VST plugins are installed
                          </div>
                        </div>
                        <ScrollArea className="h-[200px]">
                          <div className="p-1">
                            {pluginDirectories.map((directory, index) => (
                              <div key={index} className="flex items-center gap-3 p-2 hover:bg-muted/50 rounded group">
                                <Folder className="h-4 w-4 text-muted-foreground flex-shrink-0" />
                                <span className="flex-1 text-sm font-mono text-foreground truncate" title={directory}>
                                  {directory}
                                </span>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => removePluginDirectory(index)}
                                  className="h-7 w-7 p-0 opacity-0 group-hover:opacity-100 hover:bg-destructive/10 hover:text-destructive transition-opacity"
                                >
                                  <Trash2 className="h-3 w-3" />
                                </Button>
                              </div>
                            ))}
                            
                            {pluginDirectories.length === 0 && (
                              <div className="text-sm text-muted-foreground text-center py-8">
                                No plugin directories configured
                                <div className="text-xs mt-1">Add a directory above to get started</div>
                              </div>
                            )}
                          </div>
                        </ScrollArea>
                      </div>
                    </div>

                    <Separator />

                    {/* Plugin Management */}
                    <div className="space-y-3">
                      <Label className="text-sm font-medium flex items-center gap-2">
                        <div className="h-2 w-2 rounded-full bg-accent" />
                        Plugin Management
                      </Label>
                      <div className="flex gap-2">
                        <Button
                          variant="outline"
                          className="hover:bg-muted hover:text-foreground border-border bg-transparent"
                          onClick={() => scanPlugins()}
                        >
                          Scan for Plugins
                        </Button>
                        <Button
                          variant="outline"
                          className="hover:bg-muted hover:text-foreground border-border bg-transparent"
                        >
                          Reset Plugin Cache
                        </Button>
                      </div>

                      {/* Discovered Plugins List */}
                      <div className="border rounded-lg bg-card">
                        <div className="p-3 border-b bg-muted/30">
                          <div className="text-sm font-medium">Discovered Plugins</div>
                          <div className="text-xs text-muted-foreground">
                            Plugins found in configured directories
                          </div>
                        </div>
                        <ScrollArea className="h-[150px]">
                          <div className="p-1">
                            {discoveredPlugins.map((plugin, index) => (
                              <div key={index} className="flex items-center gap-3 p-2 hover:bg-muted/50 rounded">
                                <Sliders className="h-3 w-3 text-muted-foreground flex-shrink-0" />
                                <span className="flex-1 text-sm text-foreground truncate" title={plugin}>
                                  {plugin}
                                </span>
                              </div>
                            ))}
                            
                            {discoveredPlugins.length === 0 && (
                              <div className="text-sm text-muted-foreground text-center py-6">
                                No plugins discovered
                                <div className="text-xs mt-1">Add plugin directories and scan for plugins</div>
                              </div>
                            )}
                          </div>
                        </ScrollArea>
                        <div className="p-2 border-t bg-muted/10 text-center">
                          <div className="text-xs text-muted-foreground">
                            {discoveredPlugins.length} plugin{discoveredPlugins.length !== 1 ? 's' : ''} found
                          </div>
                        </div>
                      </div>
                    </div>

                    <Separator />

                    {/* Plugin Settings */}
                    <div className="space-y-4">
                      <Label className="text-sm font-medium flex items-center gap-2">
                        <Sliders className="h-4 w-4 text-accent" />
                        Plugin Settings
                      </Label>
                      <div className="space-y-3">
                        {/* <div className="flex items-center justify-between p-3 rounded-lg border bg-card">
                          <div>
                            <Label htmlFor="auto-suspend" className="text-sm font-medium">
                              Auto-suspend inactive plugins
                            </Label>
                            <p className="text-xs text-muted-foreground mt-1">
                              Automatically suspend plugins when not in use to save CPU
                            </p>
                          </div>
                          <Switch id="auto-suspend" defaultChecked className="data-[state=checked]:bg-accent" />
                        </div>
                        <div className="flex items-center justify-between p-3 rounded-lg border bg-card">
                          <div>
                            <Label htmlFor="plugin-delay" className="text-sm font-medium">
                              Compensate plugin delay
                            </Label>
                            <p className="text-xs text-muted-foreground mt-1">
                              Automatically compensate for plugin processing delay
                            </p>
                          </div>
                          <Switch id="plugin-delay" defaultChecked className="data-[state=checked]:bg-accent" />
                        </div> */}
                      </div>
                    </div>

                    <Separator />

                    {/* Plugin Performance */}
                    <div className="space-y-3">
                      <Label className="text-sm font-medium">Performance Settings</Label>
                      <div className="grid grid-cols-2 gap-4">
                        <div className="p-3 rounded-lg border bg-card">
                          <div className="text-sm font-medium">CPU Usage</div>
                          <div className="text-2xl font-bold text-accent">{cpuUsage}%</div>
                          <div className="text-xs text-muted-foreground">Current load</div>
                        </div>
                        <div className="p-3 rounded-lg border bg-card">
                          <div className="text-sm font-medium">Discovered Plugins</div>
                          <div className="text-2xl font-bold text-accent">{discoveredPlugins.length}</div>
                          <div className="text-xs text-muted-foreground">Total found</div>
                        </div>
                      </div>
                    </div>
                  </div>
                )}

                {activeTab === "general" && (
                  <div className="space-y-6">
                    {/* Practice Settings */}
                    <div className="space-y-4">
                      <Label className="text-sm font-medium flex items-center gap-2">
                        <div className="h-2 w-2 rounded-full bg-accent" />
                        Practice Settings
                      </Label>
                      <div className="space-y-3">
                        <div className="flex items-center justify-between p-3 rounded-lg border bg-card">
                          <div>
                            <Label htmlFor="metronome" className="text-sm font-medium">
                              Enable metronome by default
                            </Label>
                            <p className="text-xs text-muted-foreground mt-1">
                              Start practice sessions with metronome enabled
                            </p>
                          </div>
                          <Switch
                            id="metronome"
                            checked={enableMetronome}
                            onCheckedChange={setEnableMetronome}
                            className="data-[state=checked]:bg-accent"
                          />
                        </div>
                        <div className="flex items-center justify-between p-3 rounded-lg border bg-card">
                          <div>
                            <Label htmlFor="auto-loop" className="text-sm font-medium">
                              Auto-loop difficult sections
                            </Label>
                            <p className="text-xs text-muted-foreground mt-1">
                              Automatically detect and loop challenging parts
                            </p>
                          </div>
                          <Switch id="auto-loop" className="data-[state=checked]:bg-accent" />
                        </div>
                        <div className="flex items-center justify-between p-3 rounded-lg border bg-card">
                          <div>
                            <Label htmlFor="save-progress" className="text-sm font-medium">
                              Save practice progress
                            </Label>
                            <p className="text-xs text-muted-foreground mt-1">
                              Automatically save your practice statistics
                            </p>
                          </div>
                          <Switch id="save-progress" defaultChecked className="data-[state=checked]:bg-accent" />
                        </div>
                      </div>
                    </div>

                    <Separator />

                    {/* UI Settings */}
                    <div className="space-y-4">
                      <Label className="text-sm font-medium flex items-center gap-2">
                        <Palette className="h-4 w-4 text-accent" />
                        Interface
                      </Label>
                      <div className="space-y-3">
                        <div className="flex items-center justify-between p-3 rounded-lg border bg-card">
                          <div>
                            <Label htmlFor="dark-mode" className="text-sm font-medium">
                              Dark mode
                            </Label>
                            <p className="text-xs text-muted-foreground mt-1">Use dark theme for the interface</p>
                          </div>
                          <Switch 
                            id="dark-mode" 
                            checked={theme === "dark"} 
                            onCheckedChange={(checked) => setTheme(checked ? "dark" : "light")}
                            className="data-[state=checked]:bg-accent" 
                          />
                        </div>
                        <div className="flex items-center justify-between p-3 rounded-lg border bg-card">
                          <div>
                            <Label htmlFor="compact-ui" className="text-sm font-medium">
                              Compact interface
                            </Label>
                            <p className="text-xs text-muted-foreground mt-1">Use smaller spacing and controls</p>
                          </div>
                          <Switch id="compact-ui" className="data-[state=checked]:bg-accent" />
                        </div>
                      </div>
                    </div>

                    <Separator />

                    {/* Data Settings */}
                    <div className="space-y-3">
                      <Label className="text-sm font-medium">Data & Privacy</Label>
                      <div className="flex gap-2">
                        <Button
                          variant="outline"
                          className="hover:bg-muted hover:text-foreground border-border bg-transparent"
                        >
                          Export Practice Data
                        </Button>
                        <Button
                          variant="outline"
                          className="hover:bg-destructive/10 hover:text-destructive hover:border-destructive/20 border-border bg-transparent"
                        >
                          Clear All Data
                        </Button>
                      </div>
                      <p className="text-xs text-muted-foreground">
                        Export your practice statistics or clear all stored data
                      </p>
                    </div>

                    <Separator />

                    {/* About */}
                    <div className="space-y-3">
                      <Label className="text-sm font-medium">About</Label>
                      <div className="p-4 rounded-lg border bg-card">
                        <div className="flex items-center gap-3 mb-2">
                          <div className="h-8 w-8 rounded bg-accent flex items-center justify-center">
                            <div className="h-4 w-4 rounded-full bg-accent-foreground" />
                          </div>
                          <div>
                            <div className="font-medium">{APP_NAME}</div>
                            <div className="text-sm text-muted-foreground">Version {APP_VERSION}</div>
                          </div>
                        </div>
                        <p className="text-xs text-muted-foreground mb-2">
                          {APP_DESCRIPTION}
                        </p>
                        <p className="text-xs text-muted-foreground">
                          Created by {APP_AUTHOR}
                        </p>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </ScrollArea>

            <div className="border-t bg-background p-4 flex-shrink-0">
              <div className="flex justify-end gap-2">
                <Button
                  variant="outline"
                  onClick={() => onOpenChange(false)}
                  className="hover:bg-muted hover:text-foreground border-border"
                >
                  Cancel
                </Button>
                <Button
                  onClick={() => onOpenChange(false)}
                  className="bg-accent hover:bg-accent/90 text-accent-foreground"
                >
                  Save Changes
                </Button>
              </div>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
