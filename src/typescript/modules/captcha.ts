import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { logger } from "../utils/logger";

/** Структура данных капчи */
interface Captcha {
  type: "web" | "map";
  data: {
    url: string | null;
    base64: string | null;
  }
}

/** Модуль управления обходом капч */
class CaptchaModule {
  private settings: HTMLElement | null = null;
  private cards: HTMLElement | null = null;
  private wrappers: HTMLElement | null = null;

  /** Метод инициализации функций, связанных с обходом капчи */
  public async init(): Promise<void> {
    this.settings = document.getElementById("anti-captcha-settings");
    this.cards = document.getElementById("anti-captcha-cards");
    this.wrappers = document.getElementById("anti-captcha-wrappers");

    await listen("anti-web-captcha", (event) => {
      try {
        const payload = event.payload as { captcha_url: string; username: string; };
        const captcha_url = payload.captcha_url;
        const username = payload.username;

        this.createCaptchaCard(username, {
          type: "web",
          data: {
            url: captcha_url,
            base64: null
          }
        });
      } catch (error) {
        logger.log(`Ошибка обработки события (anti-web-captcha): ${error}`, "error");
      }
    });

    await listen("anti-map-captcha", (event) => {
      try {
        const payload = event.payload as { base64_code: string; username: string; };
        const base64 = payload.base64_code;
        const username = payload.username;

        this.createCaptchaCard(username, {
          type: "map",
          data: {
            url: null,
            base64: base64
          }
        });
      } catch (error) {
        logger.log(`Ошибка обработки события (anti-map-captcha): ${error}`, "error");
      }
    });
  }

  /** Метод активации обхода капчи */
  public enable(type: string, mode: string): void {
    if (type === "map" && mode === "manual") {
      this.settings!.style.display = "none";

      this.cards!.innerHTML = "";
      this.cards!.style.display = "flex";
    }
  }

  /** Метод очистки и выключения обхода капчи */
  public disable(): void {
    document.querySelectorAll('[wrapper="captcha-card"]').forEach(w => w.remove());
    document.querySelectorAll('[wrapper="captcha-img"]').forEach(w => w.remove());

    this.cards!.innerHTML = "";
    this.cards!.style.display = "none";

    this.wrappers!.innerHTML = "";

    this.settings!.style.display = "flex";
  }

  /** Метод создания карточки капчи */
  private createCaptchaCard(username: string, captcha: Captcha): void {
    const oldCard = document.getElementById(`bot-captcha-${username}`);

    if (oldCard) {
      oldCard.remove();
      document.getElementById(`captcha-wrapper-${username}`)?.remove();
    }

    const card = document.createElement("div");
    card.className = "bot-captcha";
    card.id = `bot-captcha-${username}`;
    card.setAttribute("wrapper", "captcha-card");
    
    card.innerHTML = `
      <div class="head">
        <div class="username">${username}</div>
      </div>

      <div class="buttons">
        <button class="min" id="solve-captcha-${username}">Решить</button>
        <button class="min" id="remove-captcha-${username}">Удалить</button>
      </div>
    `;

    this.cards?.appendChild(card);

    this.initializeCaptchaCard(username, captcha);
  }

  /** Метод инициализации карточки капчи */
  private initializeCaptchaCard(username: string, captcha: Captcha): void {
    if (captcha.type === "web") {
      document.getElementById(`solve-captcha-${username}`)?.addEventListener("click", async () => {
        const url = captcha.data.url;
        if (url && url !== "") await invoke("open_url", { url: url });
      });
    } else {
      const wrapper = document.createElement("div");
      wrapper.className = "cover";
      wrapper.id = `captcha-wrapper-${username}`;
      wrapper.setAttribute("wrapper", "captcha-img");
      
      wrapper.innerHTML = `
        <div class="panel with-header" style="margin-bottom: 20px;">
          <div class="left">
            <div class="header">Капча бота ${username}</div>
          </div>

          <div class="right">
            <button class="min" id="close-captcha-wrapper-${username}">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
                <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
              </svg>
            </button>
          </div>
        </div>

        <div class="bot-captcha-wrapper">
          <img class="captcha-img" id="captcha-img-${username}" src="data:image/png;base64,${captcha.data.base64}" draggable="false">
        </div>

        <div class="pretty-input-wrapper" style="margin-top: 20px;">
          <p class="signature">${username}</p>
          <input type="text" id="captcha-code-${username}" placeholder="Введите и отправьте код с капчи, нажав «Enter»">
        </div>
      `;

      this.wrappers?.appendChild(wrapper);

      const img = document.getElementById(`captcha-img-${username}`) as HTMLImageElement;
      img.onload = () => {
        const scale = Math.min(2, 800 / img.naturalWidth);
        img.style.width = (img.naturalWidth * scale) + "px";
        img.style.height = (img.naturalHeight * scale) + "px";
      };

      document.getElementById(`solve-captcha-${username}`)?.addEventListener("click", () => wrapper.style.display = "flex");
      document.getElementById(`close-captcha-wrapper-${username}`)?.addEventListener("click", () => wrapper.style.display = "none");

      document.getElementById(`captcha-wrapper-${username}`)?.addEventListener("keydown", async (e: Event) => {
        if ((e as KeyboardEvent).key === "Enter") {
          const input = document.getElementById(`captcha-code-${username}`) as HTMLInputElement;

          await invoke("quick_operation", { 
            operation: "send-message",
            options: {
              username: username,
              message: input.value
            }
          });

          input.value = "";
        }
      });
    }

    document.getElementById(`remove-captcha-${username}`)?.addEventListener("click", async () => {
      document.getElementById(`bot-captcha-${username}`)?.remove();
      document.getElementById(`captcha-wrapper-${username}`)?.remove();
    });
  }
}   

const captchaModule = new CaptchaModule();

export { captchaModule }
