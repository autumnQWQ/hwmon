#!/usr/bin/env swift
// hwmon macOS Overlay — transparent floating system monitor
// Polling-based hover detection (avoids NSTrackingArea crash in Swift interpreter)

import AppKit
import Foundation

let hwmonPath = CommandLine.arguments.count > 1
    ? CommandLine.arguments[1]
    : "/usr/local/bin/hwmon"

// ---- State ----
var isLocked = true
var isSingleLine = false
var isDragging = false
var wasInside = false

// ---- Layout constants ----
let pad: CGFloat = 8
let topBarH: CGFloat = 22
let iconW: CGFloat = 16
let iconGap: CGFloat = 4

// ---- Read hwmon JSON ----
func readStats() -> [String: String] {
    let task = Process()
    task.launchPath = hwmonPath
    task.arguments = ["--json"]
    let pipe = Pipe()
    task.standardOutput = pipe
    task.launch()
    task.waitUntilExit()
    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
          let cpu = json["cpu"] as? [String: Any],
          let gpu = json["gpu"] as? [String: Any],
          let mem = json["memory"] as? [String: Any] else {
        return ["--": "--"]
    }
    return [
        "cpuFreq": "\(cpu["frequency_mhz"] ?? 0)",
        "cpuUsage": String(format: "%.0f%%", (cpu["utilization_pct"] as? Double ?? 0).rounded()),
        "cpuTemp": (cpu["temperature_c"] as? Double).map { String(format: "%.0f°C", $0) } ?? "--",
        "gpuFreq": "\(gpu["frequency_mhz"] ?? 0)",
        "gpuUsage": String(format: "%.0f%%", (gpu["utilization_pct"] as? Double ?? 0).rounded()),
        "gpuTemp": (gpu["temperature_c"] as? Double).map { String(format: "%.0f°C", $0) } ?? "--",
        "memUsed": String(format: "%.1f", mem["used_gb"] as? Double ?? 0),
        "memTotal": String(format: "%.1f", mem["total_gb"] as? Double ?? 0),
        "memPct": String(format: "%.0f%%", (mem["used_pct"] as? Double ?? 0).rounded()),
        "fps": "\(json["fps"] as? Int64 ?? 0)",
    ]
}

func formatDisplay(_ s: [String: String], singleLine: Bool) -> String {
    let v = { (key: String) in s[key] ?? "--" }
    let fps = s["fps"] ?? "--"
    if singleLine {
        return "CPU \(v("cpuFreq"))MHz \(v("cpuUsage"))% \(v("cpuTemp"))  │  GPU \(v("gpuFreq"))MHz \(v("gpuUsage"))% \(v("gpuTemp"))  │  RAM \(v("memUsed"))/\(v("memTotal"))GB (\(v("memPct"))%)  │  \(fps)FPS"
    }
    return "CPU  \(v("cpuFreq"))MHz  \(v("cpuUsage"))%  \(v("cpuTemp"))\nGPU  \(v("gpuFreq"))MHz  \(v("gpuUsage"))%  \(v("gpuTemp"))\nRAM  \(v("memUsed")) / \(v("memTotal")) GB (\(v("memPct"))%)\nFPS   \(fps)"
}

// ---- Create window ----
let app = NSApplication.shared
app.setActivationPolicy(.accessory)

let screen = NSScreen.screens.first!
let vf = screen.visibleFrame

let window = NSWindow(
    contentRect: NSRect(x: vf.maxX - 260, y: vf.maxY - 120, width: 260, height: 120),
    styleMask: .borderless,
    backing: .buffered,
    defer: false
)
window.level = NSWindow.Level(rawValue: 1000)
window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
window.isOpaque = false
window.backgroundColor = NSColor.black.withAlphaComponent(0.5)
window.hasShadow = false
window.contentView?.wantsLayer = true
window.contentView?.layer?.cornerRadius = 6

// ---- Content view with drag support ----
class OverlayView: NSView {
    override func mouseDown(with event: NSEvent) {
        let p = convert(event.locationInWindow, from: nil)
        // Close button works regardless of lock state
        if closeIcon.frame.contains(p) { NSApp.terminate(nil); return }
        if isLocked {
            if lockIcon.frame.contains(p) {
                isLocked.toggle()
                lockIcon.stringValue = isLocked ? "🔒" : "🔓"
                return
            }
            if layoutIcon.frame.contains(p) {
                isSingleLine.toggle()
                layoutIcon.stringValue = isSingleLine ? "≡" : "☰"
                resizeToFit(showBar: wasInside)
                return
            }
            return
        }
        if lockIcon.frame.contains(p) {
            isLocked.toggle()
            lockIcon.stringValue = isLocked ? "🔒" : "🔓"
            return
        }
        if layoutIcon.frame.contains(p) {
            isSingleLine.toggle()
            layoutIcon.stringValue = isSingleLine ? "≡" : "☰"
            resizeToFit(showBar: wasInside)
            return
        }
        // Native drag loop
        isDragging = true
        let startOrigin = window!.frame.origin
        let startScreen = NSEvent.mouseLocation
        while true {
            guard let next = window!.nextEvent(matching: [.leftMouseDragged, .leftMouseUp]) else { break }
            if next.type == .leftMouseUp { break }
            let dx = NSEvent.mouseLocation.x - startScreen.x
            let dy = NSEvent.mouseLocation.y - startScreen.y
            window!.setFrameOrigin(NSPoint(x: startOrigin.x + dx, y: startOrigin.y + dy))
        }
        isDragging = false
        clampToScreen()
    }
}
let overlay = OverlayView(frame: window.contentView!.bounds)
overlay.autoresizingMask = [.width, .height]
window.contentView?.addSubview(overlay)

// ---- Icons ----
let lockIcon = NSTextField(frame: NSRect(x: 0, y: 0, width: iconW, height: iconW))
lockIcon.isBezeled = false; lockIcon.drawsBackground = false; lockIcon.isEditable = false
lockIcon.textColor = NSColor(white: 1, alpha: 0.9)
lockIcon.font = NSFont.systemFont(ofSize: 12); lockIcon.alignment = .center
lockIcon.stringValue = "🔒"; lockIcon.isHidden = true
overlay.addSubview(lockIcon)

let layoutIcon = NSTextField(frame: NSRect(x: 0, y: 0, width: iconW, height: iconW))
layoutIcon.isBezeled = false; layoutIcon.drawsBackground = false; layoutIcon.isEditable = false
layoutIcon.textColor = NSColor(white: 1, alpha: 0.9)
layoutIcon.font = NSFont.systemFont(ofSize: 12); layoutIcon.alignment = .center
layoutIcon.stringValue = "☰"; layoutIcon.isHidden = true
overlay.addSubview(layoutIcon)

let closeIcon = NSTextField(frame: NSRect(x: 0, y: 0, width: iconW, height: iconW))
closeIcon.isBezeled = false; closeIcon.drawsBackground = false; closeIcon.isEditable = false
closeIcon.textColor = NSColor(white: 1, alpha: 0.9)
closeIcon.font = NSFont.systemFont(ofSize: 14); closeIcon.alignment = .center
closeIcon.stringValue = "✕"; closeIcon.isHidden = true
overlay.addSubview(closeIcon)

// ---- Label ----
let label = NSTextField(frame: .zero)
label.isBezeled = false; label.drawsBackground = false
label.isEditable = false; label.isSelectable = false
label.textColor = NSColor(white: 1.0, alpha: 0.92)
label.font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
let shadow = NSShadow()
shadow.shadowColor = NSColor.black.withAlphaComponent(0.6)
shadow.shadowBlurRadius = 2.5; shadow.shadowOffset = NSSize(width: 0.5, height: -0.5)
label.shadow = shadow
overlay.addSubview(label)

// ---- Layout helpers ----
func clampToScreen() {
    let v = NSScreen.screens.first!.visibleFrame
    let f = window.frame
    var ox = f.origin.x; var oy = f.origin.y
    if ox < v.minX { ox = v.minX }
    if oy < v.minY { oy = v.minY }
    if ox + f.width > v.maxX { ox = v.maxX - f.width }
    if oy + f.height > v.maxY { oy = v.maxY - f.height }
    window.setFrameOrigin(NSPoint(x: ox, y: oy))
}

func resizeToFit(showBar: Bool) {
    label.sizeToFit()
    let fw = max(label.frame.width + pad * 2, iconW * 3 + iconGap * 2 + pad * 2)
    let fh = pad + label.frame.height + pad + (showBar ? topBarH : 0)
    var ox = window.frame.origin.x; var oy = window.frame.origin.y
    let v = NSScreen.screens.first!.visibleFrame
    if ox < v.minX { ox = v.minX }; if oy < v.minY { oy = v.minY }
    if ox + fw > v.maxX { ox = v.maxX - fw }
    if oy + fh > v.maxY { oy = v.maxY - fh }
    window.setFrame(NSRect(x: ox, y: oy, width: fw, height: fh), display: true)
    label.frame.origin = NSPoint(x: pad, y: pad)
    if showBar {
        let iconY = fh - topBarH + (topBarH - iconW) / 2
        closeIcon.frame.origin = NSPoint(x: fw - pad - iconW, y: iconY)
        lockIcon.frame.origin = NSPoint(x: fw - pad - iconW * 2 - iconGap, y: iconY)
        layoutIcon.frame.origin = NSPoint(x: fw - pad - iconW * 3 - iconGap * 2, y: iconY)
    }
}

// ---- Polling-based hover detection ----
func updateHover() {
    let inside = NSPointInRect(NSEvent.mouseLocation, window.frame)
    if inside != wasInside {
        wasInside = inside
        lockIcon.isHidden = !inside
        layoutIcon.isHidden = !inside
        closeIcon.isHidden = !inside
        if inside {
            if isLocked { NSCursor.arrow.set() } else { NSCursor.openHand.set() }
        }
    }
}

// ---- Refresh ----
func refresh() {
    guard !isDragging else { return }
    label.stringValue = formatDisplay(readStats(), singleLine: isSingleLine)
    updateHover()
    resizeToFit(showBar: wasInside)
}

// ---- Start ----
label.stringValue = formatDisplay(readStats(), singleLine: isSingleLine)
resizeToFit(showBar: wasInside)
// Initial position: top-right
let f = window.frame
window.setFrameOrigin(NSPoint(x: vf.maxX - f.width - 16, y: vf.maxY - f.height - 16))
window.makeKeyAndOrderFront(nil)

Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { _ in
    refresh()
}

app.run()
