import Module from "./module.js";
import { moduleRegistry } from "./registry.js";

class NormalizedDistance {
	inner;

	constructor(distance, lengthA, lengthB) {
		const maxLength = Math.max(lengthA, lengthB);
		this.inner = maxLength === 0 ? 1 : 1 - distance / maxLength;
	}
}

class LevenshteinDistance {
	lengthA;
	lengthB;
	inner;

	constructor(a, b) {
		this.lengthA = a.length;
		this.lengthB = b.length;

		const matrix = Array.from({ length: a.length + 1 }, () =>
			Array(b.length + 1).fill(0),
		);

		for (let i = 0; i <= a.length; i++) matrix[i][0] = i;
		for (let j = 0; j <= b.length; j++) matrix[0][j] = j;

		for (let i = 1; i <= a.length; i++) {
			for (let j = 1; j <= b.length; j++) {
				const cost = a[i - 1] === b[j - 1] ? 0 : 1;
				matrix[i][j] = Math.min(
					matrix[i - 1][j] + 1,
					matrix[i][j - 1] + 1,
					matrix[i - 1][j - 1] + cost,
				);
			}
		}

		this.inner = matrix[a.length][b.length];
	}

	normalize() {
		return new NormalizedDistance(this.inner, this.lengthA, this.lengthB);
	}
}

class SearchResult {
	inner;

	constructor(result) {
		this.inner = this.inner = [...result].sort((a, b) => a[1] - b[1]);
	}

	display(map, preview) {
		const container = document.createElement("div");

		this.inner.forEach((item) => {
			const link = document.createElement("a");
			const filename = item[0];

			link.href = filename;
			link.textContent = filename;

			link.addEventListener("mouseenter", () =>
				this.displayPreview(
					filename,
					map[filename]["title"],
					map[filename]["description"],
					map[filename]["tags"],
					preview,
				),
			);

			container.appendChild(link);
		});

		return container;
	}

	displayPreview(filename, title, description, tags, preview) {
		preview.innerHTML = "";
		preview.dataset.label = filename;
		preview.innerHTML = `
            <div>
                <span>filename</span><br>
                <p>${filename}</p>
                <span>title</span><br>
                <p>${title}</p>
                <span>description</span><br>
                <p>${description}</p>
                <span>tags</span><br>
                <p>${tags}</p>
            </div>
            `;
	}
}

class ContentMap {
	inner;

	constructor(map) {
		this.inner = map;
	}

	search(keyword) {
		let searchScoreList = Object.entries(this.inner).map((entry) => {
			const key = entry[0];
			const value = entry[1];

			const score = Math.max(
				this.evaluateString(key, keyword),
				this.evaluateObject(value, keyword),
			);

			return [key, score];
		});

		return new SearchResult(searchScoreList);
	}

	evaluateString(string, keyword) {
		return new LevenshteinDistance(string, keyword).normalize().inner;
	}

	evaluateArray(array, keyword) {
		return array.reduce(
			(max, item) => Math.max(max, this.evaluateString(item, keyword)),
			0,
		);
	}

	evaluateObject(object, keyword) {
		return Object.values(object).reduce((max, value) => {
			if (typeof value === "string") {
				return Math.max(max, this.evaluateString(value, keyword));
			}
			if (Array.isArray(value)) {
				return Math.max(max, this.evaluateArray(value, keyword));
			}
			if (value && typeof value === "object") {
				return Math.max(max, this.evaluateObject(value, keyword));
			}
			return max;
		}, 0);
	}
}

function escapeHandler(event, container) {
	if (event.key === "Escape") {
		container.classList.remove("active");
		document.removeEventListener("keydown", escapeHandler);
	}
}

function initSearch(map, input, output, preview, container) {
	const contentMap = new ContentMap(map);

	moduleRegistry.bindTap("/", () => {
		container.classList.add("active");
		input.focus();

		document.addEventListener("keydown", (event) => {
			escapeHandler(event, container);
		});
	});

	input.addEventListener("input", () => {
		output.innerHTML = "";
		output.appendChild(contentMap.search(input.value).display(map, preview));
		output.scrollTop = output.scrollHeight;
	});
}

(async () => {
	try {
		const response = await fetch("./map.json");
		const map = await response.json();

		const input = document.getElementById("search-input");
		if (!input) throw new Error("Missing element #search-input");

		const output = document.getElementById("search-output");
		if (!output) throw new Error("Missing element #search-output");

		const preview = document.getElementById("search-preview");
		if (!preview) throw new Error("Missing element #search-preview");

		const container = document.getElementById("search-container");
		if (!container) throw new Error("Missing element #search-container");

		const searchModule = new Module("search", () => {
			initSearch(map, input, output, preview, container);
		});

		moduleRegistry.register(searchModule);
	} catch (error) {
		console.error(
			"Something went wrong while mounting the Search module:",
			error,
		);
	}
})();
