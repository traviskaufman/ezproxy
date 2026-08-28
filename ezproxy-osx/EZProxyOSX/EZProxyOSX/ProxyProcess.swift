import AppKit

@MainActor
final class ProxyProcess: ObservableObject {
    @Published private(set) var isRunning = false

    private var process: Process?
    private let binaryURL = Bundle.main.url(forAuxiliaryExecutable: "ezproxy")!
    let configPath = FileManager.default.homeDirectoryForCurrentUser
        .appendingPathComponent("ezproxy.txt").path
    private let port = 5050

    init() {
        NotificationCenter.default.addObserver(
            forName: NSApplication.willTerminateNotification, object: nil, queue: .main
        ) { [weak self] _ in
            MainActor.assumeIsolated { self?.stop() }
        }
        start()
    }

    func start() {
        guard process == nil else { return }
        let process = Process()
        process.executableURL = binaryURL
        process.arguments = [configPath, "--port", String(port)]
        process.terminationHandler = { [weak self] _ in
            Task { @MainActor in self?.markStopped() }
        }
        try? process.run()
        self.process = process
        isRunning = process.isRunning
    }

    func stop() {
        guard let process else { return }
        process.terminationHandler = nil
        process.terminate()
        process.waitUntilExit()
        markStopped()
    }

    func restart() {
        stop()
        start()
    }

    private func markStopped() {
        process = nil
        isRunning = false
    }
}
