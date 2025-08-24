import Module from "./module.js";
import { moduleRegistry } from "./registry.js";
import katex from "https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.mjs";

function initMathSyntax() {
	const display = document.querySelectorAll("span[data-math-style='display']");
	const inline = document.querySelectorAll("span[data-math-style='inline']");

	inline.forEach((element) => {
		katex.render(element.textContent, element, { throwOnError: false });
	});

	display.forEach((element) => {
		katex.render(element.textContent, element, {
			displayMode: true,
			throwOnError: false,
		});
	});
}

(() => {
	const mathSyntaxModule = new Module("math-syntax", initMathSyntax);
	moduleRegistry.register(mathSyntaxModule);
})();
