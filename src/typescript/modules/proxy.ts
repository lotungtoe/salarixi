import { invoke } from "@tauri-apps/api/core";
import { logger } from "../utils/logger";
import { listen } from "@tauri-apps/api/event";

/** Модуль управления прокси и прокси-сборщика */
class ProxyModule {
  private proxyList: Array<string>;
  private proxyListEl: HTMLTextAreaElement | null;
  private counter: HTMLElement | null;
  private status: HTMLElement | null;

  constructor() {
    this.proxyList = [];
    this.proxyListEl = null;
    this.counter = null;
    this.status = null;
  }
  
  /** Метод инициализации функций, связанных с сборщиком прокси */
  public async init(): Promise<void> {
    this.proxyListEl = document.getElementById("proxy-list") as HTMLTextAreaElement;
    this.counter = document.getElementById("proxy-counter") as HTMLElement;
    this.status = document.getElementById("proxy-finder-status") as HTMLElement;

    this.proxyListEl.addEventListener("input", () => this.updateCounter());

    document.getElementById("clear-proxy-list")?.addEventListener("click", () => {
      this.proxyListEl!.value = "";
      this.updateCounter();
    });

    document.getElementById("find-proxy")?.addEventListener("click", () => this.collectProxy());

    await listen("proxy-chunk", (event) => {
      try {
        let proxies = event.payload as Array<string>;
        this.proxyList.push(...proxies);
      } catch (error) {
        logger.log(`Ошибка обработки чанка прокси: ${error}`, "error");
      }

      this.updateCounter(this.proxyList.length.toString());
    });

    await listen("proxy-chunk-end", () => {
      let proxies_str = this.proxyList.map(p => p).join("\n");
      this.proxyList = [];
      this.proxyListEl!.value = proxies_str;
    });

    this.updateCounter();
  }

  /** Метод обновления счётчика прокси */
  private updateCounter(value?: string): void {
    if (value) {
      this.counter!.innerText = value;
      return;
    }
    
    let text = this.proxyListEl!.value;

    if (!text) {
      this.counter!.innerText = "0";
      return;
    }

    let regex = /(?:\w+:\/\/)?(?:(?:\d{1,3}\.){3}\d{1,3}:\d+)/g;
    let matches = text.match(regex);
    this.counter!.innerText = matches ? matches.length.toString() : "0";
  }

  /** Метод установки статуса поиска прокси */
  private setStatus(text: string, color?: string): void {
    if (!this.status) return;
    this.status!.style.color = color ?? "#848080";
    this.status!.innerText = text;
  }
  
  /** Метод сборки прокси */
  private async collectProxy(): Promise<void> {
    try {
      this.proxyListEl!.value = "";

      const algorithm = (document.getElementById("proxy-finder_select_algorithm") as HTMLSelectElement).value;
      const country = (document.getElementById("proxy-finder_select_country") as HTMLSelectElement).value;
      const port = (document.getElementById("proxy-finder_select_port") as HTMLSelectElement).value;
      const count = (document.getElementById("proxy-finder_select_count") as HTMLInputElement).value;

      this.setStatus("Поиск прокси...");

      let success = await invoke("collect_proxies", {
        options: {
          algorithm: algorithm,
          country: country,
          port: port,
          count: count,
        }
      }) as boolean;

      if (success) {
        this.setStatus("Поиск окончен", "#0cd212ff");
      } else {
        this.setStatus("Ошибка поиска", "#cc1d1dff");
        logger.log("Ошибка сборщика прокси: Could not find a proxies", "error");
      }
    } catch (error) {
      this.setStatus("Ошибка поиска", "#cc1d1dff");
      logger.log(`Ошибка сборщика прокси: ${error}`, "error");
    } finally {
      setTimeout(() => this.setStatus("Поиск неактивен"), 2000);
    }
  }
}    

const proxyModule = new ProxyModule();

export { proxyModule }
