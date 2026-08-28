import ServiceManagement

@MainActor
final class LoginItem: ObservableObject {
    @Published var isEnabled: Bool {
        didSet { apply() }
    }

    private static let hasRegisteredKey = "LoginItem.hasRegisteredOnFirstLaunch"

    init() {
        let defaults = UserDefaults.standard
        if !defaults.bool(forKey: Self.hasRegisteredKey) {
            try? SMAppService.mainApp.register()
            defaults.set(true, forKey: Self.hasRegisteredKey)
        }
        isEnabled = SMAppService.mainApp.status == .enabled
    }

    private func apply() {
        if isEnabled {
            try? SMAppService.mainApp.register()
        } else {
            try? SMAppService.mainApp.unregister()
        }
    }
}
