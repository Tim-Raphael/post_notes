class ModuleRegistry {
    constructor() {
        this.modules = new Map();
        this.keybinds = new Map();

        document.addEventListener("keyup", (event) => {
            this.triggerKey(event.key);
        });
    }

    register(name, module) {
        if (this.modules.has(name)) {
            throw new Error(`Module "${name}" is already registered.`);
        }
        this.modules.set(name, module);
    }

    bind(key, callback) {
        this.keybinds.set(key, callback);

    }

    triggerKey(key) {
        const callback = this.keybinds.get(key);
        if (callback) callback();
    }
}

export const moduleRegistry = new ModuleRegistry();
