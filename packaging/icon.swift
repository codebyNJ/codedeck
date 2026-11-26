import AppKit
// swift packaging/icon.swift MASCOT.svg OUT.png — 1024px macOS app icon: flat white squircle tile, mascot centred.
let args = CommandLine.arguments
guard let mascot = NSImage(contentsOf: URL(fileURLWithPath: args[1])) else { fatalError("can't load svg") }
let size = 1024.0
let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: 1024, pixelsHigh: 1024, bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
// Apple's icon grid: 824pt tile inset 100pt, ~185pt corner radius.
let tile = NSRect(x: 100, y: 100, width: 824, height: 824)
let path = NSBezierPath(roundedRect: tile, xRadius: 185, yRadius: 185)
// The mascot is a full red square; the tile takes its red and the face sits
// a little inside, so the visor clears the rounded corners.
NSColor(srgbRed: 0xe4 / 255.0, green: 0x5e / 255.0, blue: 0x4d / 255.0, alpha: 1).setFill(); path.fill()
path.addClip()
let m = 760.0
mascot.draw(in: NSRect(x: (size - m) / 2, y: (size - m) / 2, width: m, height: m))
NSGraphicsContext.restoreGraphicsState()
try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: args[2]))
