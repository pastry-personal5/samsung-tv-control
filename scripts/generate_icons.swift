import AppKit
import CoreGraphics
import Foundation

// Original line artwork, rasterized at 4× the 24-point display size.
let names = [
    "power", "remote", "sources", "apps", "text_input", "settings",
    "wake", "retry", "cancel", "up", "down", "left", "right",
    "enter", "back", "home", "mute", "volume_down", "volume_up",
]
let output = URL(fileURLWithPath: CommandLine.arguments.dropFirst().first ?? "assets/icons")
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)

func draw(_ name: String, in context: CGContext) {
    let ink = CGColor(red: 0.82, green: 0.85, blue: 0.89, alpha: 1)
    context.setStrokeColor(ink)
    context.setFillColor(ink)
    context.setLineWidth(3.2)
    context.setLineCap(.round)
    context.setLineJoin(.round)

    func line(_ points: [(CGFloat, CGFloat)], closed: Bool = false) {
        guard let first = points.first else { return }
        context.beginPath()
        context.move(to: CGPoint(x: first.0, y: first.1))
        for point in points.dropFirst() {
            context.addLine(to: CGPoint(x: point.0, y: point.1))
        }
        if closed { context.closePath() }
        context.strokePath()
    }

    func circle(_ x: CGFloat, _ y: CGFloat, _ radius: CGFloat) {
        context.strokeEllipse(in: CGRect(x: x - radius, y: y - radius,
                                         width: radius * 2, height: radius * 2))
    }

    func roundRect(_ x: CGFloat, _ y: CGFloat, _ width: CGFloat, _ height: CGFloat,
                   _ radius: CGFloat = 3) {
        context.strokePath()
        context.addPath(CGPath(roundedRect: CGRect(x: x, y: y, width: width, height: height),
                               cornerWidth: radius, cornerHeight: radius, transform: nil))
        context.strokePath()
    }

    switch name {
    case "power":
        context.beginPath()
        context.addArc(center: CGPoint(x: 24, y: 23), radius: 14,
                       startAngle: .pi * 0.72, endAngle: .pi * 2.28, clockwise: false)
        context.strokePath()
        line([(24, 40), (24, 23)])
    case "remote":
        roundRect(16, 6, 16, 36, 6)
        circle(24, 32, 3)
        line([(24, 24), (24, 16)])
        line([(20, 20), (28, 20)])
    case "sources":
        line([(9, 34), (18, 34), (18, 24), (28, 24), (28, 14), (39, 14)])
        circle(9, 34, 3)
        circle(39, 14, 3)
        circle(28, 24, 3)
    case "apps":
        for x in [9.0, 25.0] {
            for y in [9.0, 25.0] { roundRect(x, y, 13, 13, 2) }
        }
    case "text_input":
        roundRect(9, 10, 30, 28, 3)
        line([(15, 31), (32, 31)])
        line([(23.5, 31), (23.5, 18)])
        line([(18, 18), (29, 18)])
    case "settings":
        circle(24, 24, 13)
        circle(24, 24, 5)
        for index in 0..<8 {
            let angle = CGFloat(index) * .pi / 4
            line([(24 + cos(angle) * 15, 24 + sin(angle) * 15),
                  (24 + cos(angle) * 19, 24 + sin(angle) * 19)])
        }
    case "wake":
        circle(24, 24, 8)
        for index in 0..<8 {
            let angle = CGFloat(index) * .pi / 4
            line([(24 + cos(angle) * 13, 24 + sin(angle) * 13),
                  (24 + cos(angle) * 18, 24 + sin(angle) * 18)])
        }
    case "retry":
        context.beginPath()
        context.addArc(center: CGPoint(x: 24, y: 24), radius: 14,
                       startAngle: .pi * 0.2, endAngle: .pi * 1.75, clockwise: false)
        context.strokePath()
        line([(34, 38), (38, 27), (27, 30)])
    case "cancel":
        circle(24, 24, 16)
        line([(18, 18), (30, 30)])
        line([(18, 30), (30, 18)])
    case "up": line([(12, 19), (24, 31), (36, 19)])
    case "down": line([(12, 29), (24, 17), (36, 29)])
    case "left": line([(30, 12), (18, 24), (30, 36)])
    case "right": line([(18, 12), (30, 24), (18, 36)])
    case "enter":
        line([(15, 25), (22, 18), (34, 31)])
        circle(24, 24, 18)
    case "back":
        line([(21, 33), (11, 24), (21, 15)])
        line([(12, 24), (30, 24)])
        context.beginPath()
        context.addArc(center: CGPoint(x: 29, y: 18), radius: 8,
                       startAngle: -.pi / 2, endAngle: .pi / 2, clockwise: false)
        context.strokePath()
    case "home":
        line([(8, 23), (24, 38), (40, 23)])
        line([(13, 25), (13, 10), (35, 10), (35, 25)])
        line([(20, 10), (20, 21), (28, 21), (28, 10)])
    case "mute", "volume_down", "volume_up":
        line([(8, 20), (15, 20), (23, 13), (23, 35), (15, 28), (8, 28)], closed: true)
        if name == "mute" {
            line([(30, 18), (39, 30)])
            line([(30, 30), (39, 18)])
        } else if name == "volume_down" {
            line([(30, 24), (40, 24)])
        } else {
            line([(30, 24), (40, 24)])
            line([(35, 19), (35, 29)])
        }
    default: fatalError("Unknown icon: \(name)")
    }
}

for name in names {
    let pixels = 96
    guard let bitmap = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: pixels,
                                       pixelsHigh: pixels, bitsPerSample: 8,
                                       samplesPerPixel: 4, hasAlpha: true,
                                       isPlanar: false, colorSpaceName: .deviceRGB,
                                       bytesPerRow: 0, bitsPerPixel: 0),
          let graphics = NSGraphicsContext(bitmapImageRep: bitmap) else {
        fatalError("Could not create bitmap for \(name)")
    }
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = graphics
    let context = graphics.cgContext
    context.clear(CGRect(x: 0, y: 0, width: pixels, height: pixels))
    context.scaleBy(x: 2, y: 2)
    draw(name, in: context)
    graphics.flushGraphics()
    NSGraphicsContext.restoreGraphicsState()
    guard let data = bitmap.representation(using: .png, properties: [:]) else {
        fatalError("Could not encode bitmap for \(name)")
    }
    try data.write(to: output.appendingPathComponent("\(name).png"))
}
