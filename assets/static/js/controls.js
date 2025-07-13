import Module from "./module.js";
import { moduleRegistry } from "./registry.js";

function initControls() {
    let scrollInterval = null;

    function startScroll(direction) {
        stopScroll();

        scrollInterval = setInterval(() => {
            window.scrollBy(0, direction);
        }, 75);
    }

    function stopScroll() {
        if (scrollInterval) {
            clearInterval(scrollInterval);
            scrollInterval = null;
        }
    }

    moduleRegistry.bindTap("j", () => window.scrollBy(0, 50));
    moduleRegistry.bindTap("k", () => window.scrollBy(0, -50));

    moduleRegistry.bindHold("j", () => startScroll(50), stopScroll);
    moduleRegistry.bindHold("k", () => startScroll(-50), stopScroll);
}

(() => {
    const controlModule = new Module("controls", initControls);
    moduleRegistry.register(controlModule);
})();
