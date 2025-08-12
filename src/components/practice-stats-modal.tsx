"use client"

import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { Separator } from "@/components/ui/separator"
import { BarChart3, Clock, Target, Repeat, TrendingUp, Calendar } from "lucide-react"

interface PracticeStatsModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function PracticeStatsModal({ open, onOpenChange }: PracticeStatsModalProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <BarChart3 className="h-5 w-5" />
            Practice Statistics
          </DialogTitle>
          <DialogDescription>Your practice session analytics</DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* Current Session */}
          <div className="space-y-3">
            <h4 className="text-sm font-medium flex items-center gap-2">
              <Clock className="h-4 w-4" />
              Current Session
            </h4>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-1">
                <p className="text-sm text-muted-foreground">Session Time</p>
                <Badge variant="outline" className="text-sm">
                  12:34
                </Badge>
              </div>
              <div className="space-y-1">
                <p className="text-sm text-muted-foreground">Loops Played</p>
                <Badge variant="outline" className="text-sm">
                  23
                </Badge>
              </div>
            </div>
          </div>

          <Separator />

          {/* Practice Goals */}
          <div className="space-y-3">
            <h4 className="text-sm font-medium flex items-center gap-2">
              <Target className="h-4 w-4" />
              Today's Goals
            </h4>
            <div className="space-y-3">
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span>Practice Time</span>
                  <span>12/30 min</span>
                </div>
                <Progress value={40} className="h-2" />
              </div>
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span>Songs Practiced</span>
                  <span>2/5</span>
                </div>
                <Progress value={40} className="h-2" />
              </div>
            </div>
          </div>

          <Separator />

          {/* Weekly Stats */}
          <div className="space-y-3">
            <h4 className="text-sm font-medium flex items-center gap-2">
              <Calendar className="h-4 w-4" />
              This Week
            </h4>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-1">
                <p className="text-sm text-muted-foreground">Total Time</p>
                <p className="text-lg font-semibold">2h 45m</p>
              </div>
              <div className="space-y-1">
                <p className="text-sm text-muted-foreground">Sessions</p>
                <p className="text-lg font-semibold">8</p>
              </div>
              <div className="space-y-1">
                <p className="text-sm text-muted-foreground">Avg. Session</p>
                <p className="text-lg font-semibold">20m</p>
              </div>
              <div className="space-y-1">
                <p className="text-sm text-muted-foreground">Streak</p>
                <div className="flex items-center gap-1">
                  <TrendingUp className="h-4 w-4 text-green-500" />
                  <p className="text-lg font-semibold">5 days</p>
                </div>
              </div>
            </div>
          </div>

          <Separator />

          {/* Most Practiced */}
          <div className="space-y-3">
            <h4 className="text-sm font-medium flex items-center gap-2">
              <Repeat className="h-4 w-4" />
              Most Practiced
            </h4>
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <div>
                  <p className="text-sm font-medium">隣室上のクラッカー</p>
                  <p className="text-xs text-muted-foreground">ZUTOMAYO</p>
                </div>
                <Badge variant="secondary">45m</Badge>
              </div>
              <div className="flex justify-between items-center">
                <div>
                  <p className="text-sm font-medium">Saturn</p>
                  <p className="text-xs text-muted-foreground">ZUTOMAYO</p>
                </div>
                <Badge variant="secondary">32m</Badge>
              </div>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
