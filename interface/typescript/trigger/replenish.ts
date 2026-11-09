import { disableParticles, enableParticles } from "../effects/particles";
import { logger } from "../utils/logger";
import { triggerRegistry } from "./registry";
import { messages } from "../utils/message";

/** Обёрточная функция для пополнения реестра триггеров */
export function replenishTriggerRegistry(): void {
	triggerRegistry.register("settings_select_nickname-type", "select", (current: HTMLSelectElement) => {
		const container = document.getElementById("custom-nickname-template-container") as HTMLInputElement;

		if (current.value === "custom") {
			container.style.display = "flex";
		} else {
			container.style.display = "none";
		}
	});

	triggerRegistry.register("settings_select_password-type", "select", (current: HTMLSelectElement) => {
		const container = document.getElementById("custom-password-template-container") as HTMLInputElement;

		if (current.value === "custom") {
			container.style.display = "flex";
		} else {
			container.style.display = "none";
		}
	});

	triggerRegistry.register("settings_select_register-mode", "select", (current: HTMLSelectElement) => {
		if (current.value === "default") {
			(document.getElementById("register-min-delay-container") as HTMLInputElement).style.display = "flex";
			(document.getElementById("register-max-delay-container") as HTMLInputElement).style.display = "flex";
			(document.getElementById("register-trigger-container") as HTMLInputElement).style.display = "none";
		} else {
			(document.getElementById("register-min-delay-container") as HTMLInputElement).style.display = "none";
			(document.getElementById("register-max-delay-container") as HTMLInputElement).style.display = "none";
			(document.getElementById("register-trigger-container") as HTMLInputElement).style.display = "flex";
		}
	});

	triggerRegistry.register("settings_select_login-mode", "select", (current: HTMLSelectElement) => {
		if (current.value === "default") {
			(document.getElementById("login-min-delay-container") as HTMLInputElement).style.display = "flex";
			(document.getElementById("login-max-delay-container") as HTMLInputElement).style.display = "flex";
			(document.getElementById("login-trigger-container") as HTMLInputElement).style.display = "none";
		} else {
			(document.getElementById("login-min-delay-container") as HTMLInputElement).style.display = "none";
			(document.getElementById("login-max-delay-container") as HTMLInputElement).style.display = "none";
			(document.getElementById("login-trigger-container") as HTMLInputElement).style.display = "flex";
		}
	});

	triggerRegistry.register("settings_chbx_use-auto-register", "checkbox", (current: HTMLInputElement) => {
		const container = document.getElementById("auto-register-input-container") as HTMLElement;

		if (current.checked) {
			container.style.display = "flex";
		} else {
			container.style.display = "none";
		}
	});

	triggerRegistry.register("settings_chbx_use-auto-login", "checkbox", (current: HTMLInputElement) => {
		const container = document.getElementById("auto-login-input-container") as HTMLElement;

		if (current.checked) {
			container.style.display = "flex";
		} else {
			container.style.display = "none";
		}
	});

	triggerRegistry.register("settings_chbx_use-auto-rejoin", "checkbox", (current: HTMLInputElement) => {
		const container = document.getElementById("auto-rejoin-input-container") as HTMLElement;

		if (current.checked) {
			container.style.display = "flex";
		} else {
			container.style.display = "none";
		}
	});

	triggerRegistry.register("settings_chbx_use-accounts", "checkbox", (current: HTMLInputElement) => {
		const openAccountsSectionBtn = document.getElementById("accounts") as HTMLButtonElement;

		const botsCountInput = document.getElementById("settings-option-bots-count-container") as HTMLDivElement;
		const basicInputContainer = document.getElementById("basic-input-container") as HTMLDivElement;

		if (current.checked) {
			openAccountsSectionBtn.style.display = "flex";

			basicInputContainer.style.display = "none";
			botsCountInput.style.display = "none";
		} else {
			openAccountsSectionBtn.style.display = "none";

			basicInputContainer.style.display = "flex";
			botsCountInput.style.display = "flex";
		}
	});

	triggerRegistry.register("settings_chbx_enable-monitoring", "checkbox", (current: HTMLInputElement) => {
		const monitoringInputContainer = document.getElementById("monitoring-input-container") as HTMLDivElement;
		const monitoringOptimizationChbx = document.getElementById("settings_chbx_monitoring-optimization") as HTMLInputElement;
		const chatMonitoringChbx = document.getElementById("settings_chbx_chat-monitoring") as HTMLInputElement;
		const inventoryMonitoringChbx = document.getElementById("settings_chbx_inventory-monitoring") as HTMLInputElement;

		if (current.checked) {
			monitoringInputContainer.style.display = "flex";
			monitoringOptimizationChbx.disabled = false;
			chatMonitoringChbx.disabled = false;
			inventoryMonitoringChbx.disabled = false;
		} else {
			monitoringInputContainer.style.display = "none";
			monitoringOptimizationChbx.disabled = true;
			chatMonitoringChbx.disabled = true;
			inventoryMonitoringChbx.disabled = true;
		}
	});

	triggerRegistry.register("settings_chbx_inventory-monitoring", "checkbox", (current: HTMLInputElement) => {
		const monitoringInventoryUpdateRateInputContainer = document.getElementById("monitoring-inventory-update-frequency-container") as HTMLDivElement;
		const monitoringItemCacheThresholdInputContainer = document.getElementById("monitoring-item-cache-threshold-container") as HTMLDivElement;

		if (current.checked) {
			monitoringInventoryUpdateRateInputContainer.style.display = "flex";
			monitoringItemCacheThresholdInputContainer.style.display = "flex";
		} else {
			monitoringInventoryUpdateRateInputContainer.style.display = "none";
			monitoringItemCacheThresholdInputContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_chat_select_mode", "select", (current: HTMLSelectElement) => {
		const chatSpammingChbxContainer = document.getElementById("chat-spamming-chbx-container") as HTMLElement;
		const chatSpammingInputContainer = document.getElementById("chat-spamming-input-container") as HTMLElement;

		const chatDefaultBtnsContainer = document.getElementById("chat-default-btns-container") as HTMLElement;
		const chatSpammingBtnsContainer = document.getElementById("chat-spamming-btns-container") as HTMLElement;

		if (current.value === "spamming") {
			chatSpammingChbxContainer.style.display = "flex";
			chatSpammingInputContainer.style.display = "flex";
			chatSpammingBtnsContainer.style.display = "flex";

			chatDefaultBtnsContainer.style.display = "none";
		} else {
			chatDefaultBtnsContainer.style.display = "flex";

			chatSpammingChbxContainer.style.display = "none";
			chatSpammingInputContainer.style.display = "none";
			chatSpammingBtnsContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_inventory_select_mode", "select", (current: HTMLSelectElement) => {
		const basicInputContainer = document.getElementById("inventory-basic-input-container") as HTMLElement;
		const basicBtnContainer = document.getElementById("inventory-basic-btn-container") as HTMLElement;
		const swapInputContainer = document.getElementById("inventory-swap-input-container") as HTMLElement;
		const swapBtnContainer = document.getElementById("inventory-swap-btn-container") as HTMLElement;
		const interactionBtnContainer = document.getElementById("inventory-interaction-btn-container") as HTMLElement;

		if (current.value === "basic") {
			basicBtnContainer.style.display = "flex";
			basicInputContainer.style.display = "flex";
			swapInputContainer.style.display = "none";
			swapBtnContainer.style.display = "none";
			interactionBtnContainer.style.display = "none";
		} else if (current.value === "swap") {
			swapInputContainer.style.display = "flex";
			swapBtnContainer.style.display = "flex";
			basicInputContainer.style.display = "flex";
			basicBtnContainer.style.display = "none";
			interactionBtnContainer.style.display = "none";
		} else if (current.value === "interaction") {
			swapInputContainer.style.display = "none";
			swapBtnContainer.style.display = "none";
			basicInputContainer.style.display = "none";
			basicBtnContainer.style.display = "none";
			interactionBtnContainer.style.display = "flex";
		}
	});

	triggerRegistry.register("module_movement_select_mode", "select", (current: HTMLSelectElement) => {
		const selectMovementDirectionContainer = document.getElementById("select-movement-direction-container") as HTMLElement;
		const movementPathfinderInputContainer = document.getElementById("movement-pathfinder-input-container") as HTMLElement;
		const movementImpulsivenessContainer = document.getElementById("movement-impulsiveness-container") as HTMLElement;

		if (current.value === "default") {
			selectMovementDirectionContainer.style.display = "flex";
			movementPathfinderInputContainer.style.display = "none";
			movementImpulsivenessContainer.style.display = "flex";
		} else {
			selectMovementDirectionContainer.style.display = "none";
			movementPathfinderInputContainer.style.display = "flex";
			movementImpulsivenessContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_flight_select_settings", "select", (current: HTMLSelectElement) => {
		const manualSettingsContainer = document.getElementById("flight-manual-settings-container") as HTMLElement;
		const selectAntiCheatContainer = document.getElementById("flight-select-anti-cheat-container") as HTMLElement;

		if (current.value === "adaptive") {
			manualSettingsContainer.style.display = "none";
			selectAntiCheatContainer.style.display = "grid";
		} else {
			manualSettingsContainer.style.display = "flex";
			selectAntiCheatContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_killaura_select_target", "select", (current: HTMLSelectElement) => {
		const customGoalInputContainer = document.getElementById("killaura-custom-goal-input-container") as HTMLElement;

		if (current.value === "custom") {
			customGoalInputContainer.style.display = "flex";
		} else {
			customGoalInputContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_killaura_select_settings", "select", (current: HTMLSelectElement) => {
		const manualSettingsContainer = document.getElementById("killaura-manual-settings-container") as HTMLElement;

		if (current.value === "adaptive") {
			manualSettingsContainer.style.display = "none";
		} else {
			manualSettingsContainer.style.display = "flex";
		}
	});

	triggerRegistry.register("module_killaura_chbx_use-auto-weapon", "checkbox", (current: HTMLInputElement) => {
		const selectWeaponContainer = document.getElementById("select-killaura-weapon-container") as HTMLElement;
		const weaponSlotContainer = document.getElementById("killaura-weapon-slot-container") as HTMLElement;

		if (current.checked) {
			selectWeaponContainer.style.display = "grid";
			weaponSlotContainer.style.display = "none";
		} else {
			selectWeaponContainer.style.display = "none";
			weaponSlotContainer.style.display = "flex";
		}
	});

	triggerRegistry.register("module_killaura_chbx_use-chase", "checkbox", (current: HTMLInputElement) => {
		const chaseSettingsContainer = document.getElementById("killaura-chase-settings-container") as HTMLElement;

		if (current.checked) {
			chaseSettingsContainer.style.display = "flex";
		} else {
			chaseSettingsContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_bow-aim_select_target", "select", (current: HTMLSelectElement) => {
		const customGoalInputContainer = document.getElementById("bow-aim-custom-goal-input-container") as HTMLElement;

		if (current.value === "custom") {
			customGoalInputContainer.style.display = "flex";
		} else {
			customGoalInputContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_miner_select_mode", "select", (current: HTMLSelectElement) => {
		const tunnelSelectContainer = document.getElementById("miner-tunnel-select-container") as HTMLElement;
		const lookSelectContainer = document.getElementById("miner-look-select-container") as HTMLElement;

		if (current.value === "default") {
			tunnelSelectContainer.style.display = "none";
			lookSelectContainer.style.display = "none";
		} else {
			tunnelSelectContainer.style.display = "grid";
			lookSelectContainer.style.display = "grid";
		}
	});

	triggerRegistry.register("module_farmer_chbx_use-auto-tool", "checkbox", (current: HTMLInputElement) => {
		const toolSlotInputContainer = document.getElementById("farmer-tool-slot-input-container") as HTMLDivElement;

		if (current.checked) {
			toolSlotInputContainer.style.display = "none";
		} else {
			toolSlotInputContainer.style.display = "flex";
		}
	});

	triggerRegistry.register("journal_chbx_show-system-logs", "checkbox", (current: HTMLInputElement) => logger.setVisibility("system", current.checked));
	triggerRegistry.register("journal_chbx_show-extended-logs", "checkbox", (current: HTMLInputElement) => logger.setVisibility("extended", current.checked));

	triggerRegistry.register("interface_select_particles", "select", (current: HTMLSelectElement) => {
		try {
			if (current.value === "enable") {
				enableParticles();
			} else {
				disableParticles();
			}
		} catch (error) {
			logger.log(`Lỗi thay đổi trạng thái hạt hiệu ứng: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_global-font-family", "select", (current: HTMLSelectElement) => {
		try {
			const root = document.documentElement;

			switch (current.value) {
				case "inter":
					root.style.setProperty("--global-font-family", `"Inter", sans-serif`);
					break;
				case "jetbrains-mono":
					root.style.setProperty("--global-font-family", `"JetBrains Mono", "Fira Code", monospace`);
					break;
				case "segoe-ui":
					root.style.setProperty("--global-font-family", `"Segoe UI", Tahoma, Geneva, Verdana, sans-serif`);
					break;
			}
		} catch (error) {
			logger.log(`Lỗi thay đổi phông chữ: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_show-messages", "select", (current: HTMLSelectElement) => {
		try {
			messages.visibility(current.value);

			document.querySelectorAll<HTMLElement>(".message").forEach(m => {
				if (current.value === "hide") {
					m.style.display = "none";
				} else {
					m.style.display = "flex";
				}
			});
		} catch (error) {
			logger.log(`Lỗi thay đổi trạng thái thông báo: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_show-panel-icons", "select", (current: HTMLSelectElement) => {
		try {
			const display = current.value === "hide" ? "none" : "flex";
			document.querySelectorAll<SVGElement>(".dashboard button svg").forEach(i => i.style.display = display);
		} catch (error) {
			logger.log(`Lỗi thay đổi trạng thái biểu tượng trên thanh bên: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_panel-font-family", "select", (current: HTMLSelectElement) => {
		try {
			const root = document.documentElement;

			switch (current.value) {
				case "inter":
					root.style.setProperty("--panel-font-family", `"Inter", sans-serif`);
					root.style.setProperty("--panel-text-margin-top", "1px");
					break;
				case "jetbrains-mono":
					root.style.setProperty("--panel-font-family", `"JetBrains Mono", "Fira Code", monospace`);
					root.style.setProperty("--panel-text-margin-top", "2px");
					break;
				case "segoe-ui":
					root.style.setProperty("--panel-font-family", `"Segoe UI", Tahoma, Geneva, Verdana, sans-serif`);
					root.style.setProperty("--panel-text-margin-top", "-1px");
					break;
				case "courier-new":
					root.style.setProperty("--panel-font-family", `"Courier New", Courier, monospace`);
					root.style.setProperty("--panel-text-margin-top", "4px");
					break;
				case "trebuchet-ms":
					root.style.setProperty("--panel-font-family", `"Trebuchet MS", "Lucida Sans Unicode", "Lucida Grande", "Lucida Sans", Arial, sans-serif`);
					root.style.setProperty("--panel-text-margin-top", "1px");
					break;
			}
		} catch (error) {
			logger.log(`Lỗi thay đổi phông chữ thanh bên: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_panel-font-size", "select", (current: HTMLSelectElement) => {
		try {
			const root = document.documentElement;
			root.style.setProperty("--panel-font-size", current.value);
		} catch (error) {
			logger.log(`Lỗi thay đổi cỡ chữ thanh bên: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_panel-internal-gap", "select", (current: HTMLSelectElement) => {
		try {
			const root = document.documentElement;
			root.style.setProperty("--panel-btn-gap", current.value);
		} catch (error) {
			logger.log(`Lỗi thay đổi khoảng đệm trong của nút thanh bên: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_panel-separators", "select", (current: HTMLSelectElement) => {
		try {
			const display = current.value === "hide" ? "none" : "flex";
			document.querySelectorAll<HTMLElement>(".dashboard .splitter").forEach(s => s.style.display = display);
		} catch (error) {
			logger.log(`Lỗi thay đổi hiển thị dấu phân cách thanh bên: ${error}`, "error");
		}
	});
}

