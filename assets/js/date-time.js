import Module from "./module.js";
import { moduleRegistry } from "./registry.js";

function initDate(dateContainer) {
    let now = new Date();

    const d = padZero(now.getDate());
    const m = padZero(now.getMonth() + 1);
    const y = now.getFullYear();

    dateContainer.textContent = `DATE: ${y}-${m}-${d}`;
}

function initClock(clockContainer) {
    const now = new Date();

    const h = padZero(now.getHours());
    const m = padZero(now.getMinutes());
    const s = padZero(Math.floor(now.getSeconds() / 5) * 5);

    clockContainer.textContent = `TIME: ${h}:${m}:${s}`;

    setTimeout(() => {
        initClock(clockContainer);
    }, 5000);
}

function padZero(i) {
    return i < 10 ? "0" + i : i;
}

(() => {
    try {
        const dateContainer = document.getElementById("date-container");
        if (!dateContainer) throw new Error("Missing element #date-container");

        const clockContainer = document.getElementById("clock-container");
        if (!clockContainer) throw new Error("Missing element #clock-container");

        const DateTime = new Module("date-time", () => {
            initDate(dateContainer);
            initClock(clockContainer);
        });

        moduleRegistry.register(DateTime);
    } catch (error) {
        console.error("Something went wrong while mounting the Date-Time module:", error);

    }
})();
