import AppKit
import AVFoundation
import QuartzCore

final class WelcomeWindow: NSWindow {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }
}

final class WelcomeActions: NSObject {
    @objc func start(_ sender: Any?) {
        NSApp.terminate(sender)
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.regular)

let screen = NSScreen.main!
let window = WelcomeWindow(
    contentRect: screen.frame, styleMask: .borderless, backing: .buffered, defer: false, screen: screen
)
window.setFrame(screen.frame, display: true)
window.level = .mainMenu + 1
window.title = "santiaguito.cl"

let content = window.contentView!
content.wantsLayer = true
content.layer!.backgroundColor = NSColor.black.cgColor

let player = AVQueuePlayer()
let video = AVPlayerItem(url: URL(fileURLWithPath: CommandLine.arguments[1]))
let runtime = URL(fileURLWithPath: CommandLine.arguments[2], isDirectory: true)
let looper = AVPlayerLooper(player: player, templateItem: video)
let background = AVPlayerLayer(player: player)
background.frame = content.bounds
background.videoGravity = .resizeAspectFill
content.layer!.addSublayer(background)

let shade = CAGradientLayer()
shade.frame = content.bounds
shade.colors = [
    NSColor.black.withAlphaComponent(0.87).cgColor,
    NSColor.black.withAlphaComponent(0.53).cgColor,
    NSColor.clear.cgColor
]
shade.locations = [0, 0.39, 0.83]
shade.startPoint = CGPoint(x: 0, y: 0)
shade.endPoint = CGPoint(x: 0, y: 1)
content.layer!.addSublayer(shade)

let title = NSTextField(labelWithString: "santiaguito.cl")
title.font = NSFont.monospacedSystemFont(ofSize: min(screen.frame.width * 0.075, 116), weight: .bold)
title.textColor = .white

let actions = WelcomeActions()
let start = NSButton(title: "Comenzar", target: actions, action: #selector(WelcomeActions.start(_:)))
start.bezelStyle = .rounded
start.controlSize = .large
start.font = NSFont.systemFont(ofSize: 25, weight: .semibold)
start.keyEquivalent = "\r"
start.isEnabled = false

let controls = NSStackView(views: [title, start])
controls.orientation = .vertical
controls.alignment = .centerX
controls.spacing = 23
controls.translatesAutoresizingMaskIntoConstraints = false
content.addSubview(controls)
NSLayoutConstraint.activate([
    controls.centerXAnchor.constraint(equalTo: content.centerXAnchor),
    controls.centerYAnchor.constraint(equalTo: content.centerYAnchor, constant: -screen.frame.height * 0.10),
    start.widthAnchor.constraint(equalToConstant: 180),
    start.heightAnchor.constraint(greaterThanOrEqualToConstant: 47)
])

let splash = NSView(frame: content.bounds)
splash.autoresizingMask = [.width, .height]
splash.wantsLayer = true
splash.layer!.backgroundColor = NSColor(srgbRed: 0.05, green: 0.08, blue: 0.10, alpha: 1).cgColor
let wordmark = NSTextField(labelWithString: title.stringValue)
wordmark.font = NSFont.monospacedSystemFont(ofSize: 34, weight: .semibold)
wordmark.textColor = .white
let progress = NSProgressIndicator()
progress.style = .bar
progress.isIndeterminate = false
progress.maxValue = 2
let loading = NSStackView(views: [wordmark, progress])
loading.orientation = .vertical
loading.alignment = .centerX
loading.spacing = 14
loading.translatesAutoresizingMaskIntoConstraints = false
splash.addSubview(loading)
content.addSubview(splash)
NSLayoutConstraint.activate([
    loading.centerXAnchor.constraint(equalTo: splash.centerXAnchor),
    loading.centerYAnchor.constraint(equalTo: splash.centerYAnchor),
    progress.widthAnchor.constraint(equalTo: wordmark.widthAnchor)
])

window.defaultButtonCell = start.cell as? NSButtonCell
let escape = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
    if event.keyCode == 53 { exit(130) }
    return event
}
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
player.play()
Timer.scheduledTimer(withTimeInterval: 0.08, repeats: true) { timer in
    if FileManager.default.fileExists(atPath: runtime.appendingPathComponent("engine-starting").path) {
        progress.doubleValue = 1
    }
    if background.isReadyForDisplay && FileManager.default.fileExists(atPath: runtime.appendingPathComponent("ready").path) {
        splash.removeFromSuperview()
        start.isEnabled = true
        window.makeFirstResponder(start)
        timer.invalidate()
    }
}
withExtendedLifetime((window, looper, actions, escape)) {
    app.run()
}
