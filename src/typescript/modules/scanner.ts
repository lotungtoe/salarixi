import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { messages } from "../utils/message";
import { logger } from "../utils/logger";

interface ServerMeta {
	address: string;
	icon: string | null;
	version: number;
}

/** Модуль сканирования хостов на наличие Minecraft серверов */
class ScannerModule {
	private serverList: HTMLDivElement | null = null;

	/** Метод инициализации модуля */
	public async init(): Promise<void> {
		this.serverList = document.getElementById("scanner-server-list") as HTMLDivElement;

		const ipRangeInput = document.getElementById("scanner-ip-range") as HTMLInputElement;
		const taskCountInput = document.getElementById("scanner-task-count") as HTMLInputElement;
		const targetPortInput = document.getElementById("scanner-target-port") as HTMLInputElement;
		const timeoutInput = document.getElementById("scanner-timeout") as HTMLInputElement;

		document.getElementById("start-network-scanning")?.addEventListener("click", async () => {
			const range = ipRangeInput.value;
			if (range === "") return;

			if (this.serverList) this.serverList.innerHTML = "";

			const taskCount = parseInt(taskCountInput.value);
			const targetPort = parseInt(targetPortInput.value === "" ? "25565" : targetPortInput.value);
			const timeout = parseInt(timeoutInput.value);

			await invoke("start_network_scanning", {
				range: range,
				taskCount: taskCount < 1 ? 10 : taskCount,
				targetPort: targetPort,
				timeout: timeout < 1 ? 8000 : timeout,
			});
			
			messages.message("Сканер", `Сканирование серверов в диапазоне ${range}...`);
			logger.log(`Сканирование серверов в диапазоне ${range}...`, "system");
		});

		document.getElementById("stop-network-scanning")?.addEventListener("click", async () => {
			await invoke("stop_network_scanning");
			messages.message("Сканер", "Сканирование серверов остановлено");
			logger.log("Сканирование серверов остановлено", "system");
		});

		await listen<ServerMeta>("scanner:push-server", (e) => this.pushServerToList(e.payload));

		const scannerStatus = document.getElementById("scanner-status") as HTMLSpanElement;
		await listen<number>("scanner:status", (e) => {
			scannerStatus.innerText = e.payload === 0 ? "inactive" : "active";
		});

		const scannedHosts = document.getElementById("scanner-hosts-scanned") as HTMLSpanElement;
		await listen<Array<number>>("scanner:hosts-scanned", (e) => {
			const scanned = e.payload[0];
			const total = e.payload[1];
			scannedHosts.innerText = `${scanned}/${total}`;
		});

		const foundServers = document.getElementById("scanner-servers-found") as HTMLSpanElement;
		await listen<number>("scanner:servers-found", (e) => {
			foundServers.innerText = e.payload.toString();
		});
	}

	/** Метод добавления сервера в список */
	private pushServerToList(meta: ServerMeta): void {
		if (!this.serverList) return;

		const card = document.createElement("div");
		card.className = "server";
		card.innerHTML = `
			${meta.icon ? `<img src=\"${meta.icon}\" draggable="false">` : ""}
			<div class="info">
				<label class="ip">${meta.address}</label>
				<label class="version">${meta.version}</label>
			</div>
		`;

		this.serverList.appendChild(card);
	}
}

const scannerModule = new ScannerModule();

export { scannerModule }
