import Foundation

final class ConfigWatcher {
    private let path: String
    private let onChange: () -> Void
    private let queue = DispatchQueue(label: "ezproxy.config-watcher")
    private var source: DispatchSourceFileSystemObject?
    private var pendingChange: DispatchWorkItem?

    init(path: String, onChange: @escaping () -> Void) {
        self.path = path
        self.onChange = onChange
        watch()
    }

    private func watch() {
        let fd = open(path, O_EVTONLY)
        guard fd >= 0 else {
            queue.asyncAfter(deadline: .now() + 1) { [weak self] in self?.watch() }
            return
        }
        let source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd, eventMask: [.write, .delete, .rename], queue: queue
        )
        source.setEventHandler { [weak self] in
            guard let self else { return }
            let replaced = !source.data.intersection([.delete, .rename]).isEmpty
            self.scheduleChange()
            if replaced {
                source.cancel()
                self.watch()
            }
        }
        source.setCancelHandler { close(fd) }
        source.resume()
        self.source = source
    }

    private func scheduleChange() {
        pendingChange?.cancel()
        let work = DispatchWorkItem { [onChange] in DispatchQueue.main.async(execute: onChange) }
        pendingChange = work
        queue.asyncAfter(deadline: .now() + .milliseconds(300), execute: work)
    }
}
