import { controlWrappers, globalWrappers, latestControlWrapper } from "../main";

/**
 * Функция переключения видимости глобальных обёрток.
 * 
 * При открытии "control-container" функция показывает `latestControlWrapper` (последняя открытая обёртка).
 * Если `latestControlWrapper` не инициализирован, показывается "control-chat-container".
 */
const switchGlobalWrapper = (id: string): void => {
	// Hide all global wrappers first
	globalWrappers.forEach(w => w && w.el.style.display = "none");

	// Show the requested container if it exists in the DOM
	const targetEl = document.getElementById(id);
	if (targetEl) {
		targetEl.style.display = "flex";
	} else if (id === "control-container") {
		// Fallback: show latestControlWrapper or control-chat-container
		if (latestControlWrapper && latestControlWrapper.parentElement) latestControlWrapper.style.display = "flex";
		else (document.getElementById("control-chat-container") as HTMLElement).style.display = "flex";
	}
}

/** Функция переключения видимости обёрток управления */
const switchControlWrapper = (id: string): void => controlWrappers.forEach(c => c && c.id === id ? c.el.style.display = "flex" : c.el.style.display = "none");

export {
	switchGlobalWrapper,
	switchControlWrapper
}
