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

let controls = NSStackView(views: [title, start])
controls.orientation = .vertical
controls.alignment = .leading
controls.spacing = 23
controls.translatesAutoresizingMaskIntoConstraints = false
content.addSubview(controls)
NSLayoutConstraint.activate([
    controls.leadingAnchor.constraint(equalTo: content.leadingAnchor, constant: screen.frame.width * 0.08),
    controls.bottomAnchor.constraint(equalTo: content.bottomAnchor, constant: screen.frame.height * 0.12),
    start.widthAnchor.constraint(equalToConstant: 180),
    start.heightAnchor.constraint(greaterThanOrEqualToConstant: 47)
])

window.defaultButtonCell = start.cell as? NSButtonCell
let escape = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
    if event.keyCode == 53 { exit(130) }
    return event
}
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
player.play()
withExtendedLifetime((window, looper, actions, escape)) {
    app.run()
}
