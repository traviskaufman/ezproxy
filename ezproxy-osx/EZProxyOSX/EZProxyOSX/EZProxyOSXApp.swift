import SwiftUI

@main
struct EZProxyOSXApp: App {
    @StateObject private var proxy = ProxyProcess()

    var body: some Scene {
        MenuBarExtra {
            Text(proxy.isRunning ? "EZProxy: Running" : "EZProxy: Stopped")
            Divider()
            Button("Start") { proxy.start() }.disabled(proxy.isRunning)
            Button("Stop") { proxy.stop() }.disabled(!proxy.isRunning)
            Button("Restart") { proxy.restart() }.disabled(!proxy.isRunning)
            Divider()
            Button("Quit") { NSApplication.shared.terminate(nil) }
        } label: {
            Image("MenuBarIcon")
        }
    }
}
