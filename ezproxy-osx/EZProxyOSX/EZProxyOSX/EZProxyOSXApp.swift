import SwiftUI

@main
struct EZProxyOSXApp: App {
    @StateObject private var proxy = ProxyProcess()
    @StateObject private var loginItem = LoginItem()
    private let configWatcher: ConfigWatcher

    init() {
        let proxy = ProxyProcess()
        _proxy = StateObject(wrappedValue: proxy)
        configWatcher = ConfigWatcher(path: proxy.configPath) { proxy.restart() }
    }

    var body: some Scene {
        MenuBarExtra {
            Text(proxy.isRunning ? "EZProxy: Running" : "EZProxy: Stopped")
            Divider()
            Button("Start") { proxy.start() }.disabled(proxy.isRunning)
            Button("Stop") { proxy.stop() }.disabled(!proxy.isRunning)
            Button("Restart") { proxy.restart() }.disabled(!proxy.isRunning)
            Divider()
            Toggle("Launch at Login", isOn: $loginItem.isEnabled)
            Divider()
            Button("Quit") { NSApplication.shared.terminate(nil) }
        } label: {
            Image("MenuBarIcon")
        }
    }
}
