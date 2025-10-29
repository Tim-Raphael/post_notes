import Module from "./module.js";
import { moduleRegistry } from "./registry.js";

function createOrUseContainer(link, preview) {
	let container = document.getElementById(`hover-${link}`);

	if (container) return container;

	container = document.createElement("div");
	container.classList.add("preview");
	container.classList.add("container");
	container.id = `hover-${link}`;
	container.innerHTML = `
				<span>${link}</span>
				<div class="h1">${preview.title}</div>
				<p>${preview.description}</p>
			`;

	document.body.append(container);

	return container;
}

function calculatePosition(target) {
	const boundingClientRect = target.getBoundingClientRect();
	return [boundingClientRect.x, boundingClientRect.y];
}

function initLinkPreview(map) {
	const links = document.querySelectorAll("a");

	links.forEach((link) => {
		link.addEventListener("mouseover", (event) => {
			const target = event.target;
			const link = target.pathname.split("/").pop();
			const preview = map[link];

			if (!preview) return;

			const container = createOrUseContainer(link, preview);
			const [x, y] = calculatePosition(target);

			container.style.left = `${x}px`;
			container.style.top = `${y}px`;
			container.classList.remove("hidden");
		});

		link.addEventListener("mouseout", (event) => {
			const link = event.target.pathname.split("/").pop();
			const container = document.getElementById(`hover-${link}`);

			if (!container) return;

			container.classList.add("hidden");
		});
	});
}

(async () => {
	try {
		const response = await fetch("./map.json");
		const map = await response.json();

		const linkPreviewModule = new Module("link-preview", () => {
			initLinkPreview(map);
		});

		moduleRegistry.register(linkPreviewModule);
	} catch (error) {
		console.error(
			"Something went wrong while mounting the Link-Preview module:",
			error,
		);
	}
})();
