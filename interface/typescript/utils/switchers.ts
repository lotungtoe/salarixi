import { controlWrappers, globalWrappers, latestControlWrapper } from "../main";

/**
 * Функция переключения видимости глобальных обёрток.
 * 
 * При открытии "control-container" функция показывает `latestControlWrapper` (последняя открытая обёртка).
 * Если `latestControlWrapper` не инициализирован, показывается "control-chat-container".
 */
const switchGlobalWrapper = (id: string): void => {
	// Hide all global wrappers first
	for (const w of globalWrappers) {
		if (w && w.el) w.el.style.display = "none";
	}

	// Show the requested container if it exists in the DOM
	const targetEl = document.getElementById(id);
	if (targetEl) {
		targetEl.style.display = "flex";
	} else if (id === "control-container") {
		// Fallback: show latestControlWrapper or control-chat-container
		if (latestControlWrapper && latestControlWrapper.parentElement) latestControlWrapper.style.display = "flex";
		else (document.getElementById("control-chat-container") as HTMLElement).style.display = "flex";
	}
};

/** Функция переключения видимости обёрток управления */
const switchControlWrapper = (id: string): void => {
	for (const c of controlWrappers) {
		if (c && c.id === id) c.el.style.display = "flex";
		else if (c) c.el.style.display = "none";
	}
};

export {
	switchGlobalWrapper,
	switchControlWrapper
}