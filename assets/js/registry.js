// TODO! Split module and keybind registery
class ModuleRegistry {
    modules;
    tapCallbacks;
    holdStartCallbacks;
    holdStopCallbacks;
    keyState;
    holdThreshold;

    constructor() {
        this.modules = new Map();
        this.tapCallbacks = new Map();
        this.holdStartCallbacks = new Map();
        this.holdStopCallbacks = new Map();
        this.keyState = new Map();
        this.holdThreshold = 300;

        this.setupListeners();
    }

    register(module) {
        if (this.modules.has(module.id)) {
            throw new Error(`Module "${module.id}" is already registered.`);
        }
        this.modules.set(module.id, module.module);
        module.run();
    }

    bindTap(key, callback) {
        this.tapCallbacks.set(key, callback);
    }

    bindHold(key, onStart, onStop) {
        this.holdStartCallbacks.set(key, onStart);
        if (onStop) {
            this.holdStopCallbacks.set(key, onStop);
        }
    }

    setupListeners() {
        document.addEventListener("keydown", (event) => {
            const key = event.key;
            if (this.keyState.has(key)) return; // already pressed

            const startTime = Date.now();
            const timeoutId = setTimeout(() => {
                const state = this.keyState.get(key);
                if (!state) return;
                state.holdFired = true;

                const onStart = this.holdStartCallbacks.get(key);
                if (onStart) onStart();
            }, this.holdThreshold);

            this.keyState.set(key, {
                startTime,
                timeoutId,
                holdFired: false
            });
        });

        document.addEventListener("keyup", (event) => {
            const key = event.key;
            const state = this.keyState.get(key);
            if (!state) return;

            clearTimeout(state.timeoutId);

            if (state.holdFired) {
                const onStop = this.holdStopCallbacks.get(key);
                if (onStop) onStop();
            } else {
                const onTap = this.tapCallbacks.get(key);
                if (onTap) onTap();
            }

            this.keyState.delete(key);
        });
    }
}

export const moduleRegistry = new ModuleRegistry();
