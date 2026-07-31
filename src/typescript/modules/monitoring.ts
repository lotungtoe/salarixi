import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { logger } from "../utils/logger";
import { date } from "../utils/date";
import { processActivity } from "../main";

/** Структура информации о боте */
interface BotProfile {
  status: any;
  password: string | null;
  proxy: { 
    ip_address: string | null;
    proxy: string | null;
    username: string | null;
    password: string | null;
  };
  ping: number;
  health: number;
  registered: boolean;
  captcha_caught: boolean;
  group: string;
}

/** Модуль управления мониторингом */
class MonitoringModule {
  private usernameList: string[] = [];

  private statusText: HTMLElement | null = null;
  private cards: HTMLElement | null = null;
  private wrappers: HTMLElement | null = null;

  private chatMessageCounter: Record<string, number> = {};
  private chatHistoryFilters: Record<string, string> = {};

  public maxChatHistoryLength: number = 0;

  /** Метод инициализации функций, связанных с мониторингом */
  public async init(): Promise<void> {
    this.statusText = document.getElementById("monitoring-status-text");
    this.cards = document.getElementById("bot-cards-container");
    this.wrappers = document.getElementById("bot-wrappers-container");

    this.statusText!.innerText = "Объекты ботов отсутствуют";
    this.statusText!.style.display = "flex";

    await listen("monitoring:chat-message", (event) => {
      try {
        const payload = event.payload as { receiver: string; message: string; };
        const receiver = payload.receiver;
        const message = payload.message;

        if (!this.chatHistoryFilters[receiver]) this.chatHistoryFilters[receiver] = "all";
        if (!this.filterMessage(this.chatHistoryFilters[receiver], message)) return;

        const chat = document.getElementById(`monitoring-chat-content-${receiver}`);
        if (!chat) return;

        const line = document.createElement("div");
        line.className = "line";
        line.setAttribute("monitoring-message", receiver);

        line.innerHTML = `
          <div class="time">${date()}</div>
          <div class="msg">${message}</div>
        `;

        chat.appendChild(line);

        chat.scrollTo({
          top: chat.scrollHeight,
          behavior: "smooth"
        });

        if (!this.chatMessageCounter[receiver]) {
          this.chatMessageCounter[receiver] = 1;
        } else {
          this.chatMessageCounter[receiver]++;
          if (this.chatMessageCounter[receiver] > this.maxChatHistoryLength) {
            this.chatMessageCounter[receiver]--;
            chat.firstChild?.remove();
          }
        }
      } catch (error) {
        logger.log(`Ошибка мониторинга (receive-payload): ${error}`, "error");
      }
    });
  }

  /** Метод переключения состояния мониторинга на ожидание */
  public wait(): void {
    this.statusText!.innerText = "Ожидание активных ботов...";
    this.cards!.innerHTML = "";
  }

  /** Метод активации мониторинга */
  public enable(delay: number): void {
    try {
      let isFirst = true;

      const interval = setInterval(async () => {
        if (!processActivity) {
          clearInterval(interval);
          return;
        }

        try {
          const profiles = await invoke<Record<string, BotProfile>>("get_bot_profiles");

          for (const username in profiles) {
            if (isFirst) {
              this.statusText!.style.display = "none";
              this.cards!.style.display = "flex";
              isFirst = false;
            }

            const profile = profiles[username];

            this.usernameList.includes(username) ? this.updateBotCard(username, profile) : this.createBotCard(username, profile);
          }
        } catch (error) {
          logger.log(`Ошибка мониторинга профилей: ${error}`, "error");
        }
      }, delay);
    } catch (error) {
      logger.log(`Ошибка инициализации мониторинга: ${error}`, "error");
    }
  }

  /** Метод очистки и выключения мониторинга */
  public disable(): void {
    document.querySelectorAll('[wrapper="bot-chat"]').forEach(w => w.remove());
    document.querySelectorAll('[wrapper="bot-card"]').forEach(w => w.remove());

    this.cards!.innerHTML = "";
    this.cards!.style.display = "none";
    this.wrappers!.innerHTML = "";

    this.chatMessageCounter = {};
    this.chatHistoryFilters = {};
    this.usernameList = [];

    this.statusText!.innerText = "Объекты ботов отсутствуют";
    this.statusText!.style.display = "flex";
  }

  /** Метод создания карточки бота */
  private createBotCard(username: string, profile: BotProfile): void {
    const steveIconPath = document.getElementById("steve-img") as HTMLImageElement;

    const groupNameExamples = [
      "killaura", "bow_aim", "AutoFarm", 
      "AFK", "Travelers", "miner", 
      "Stealer", "Farmer", "Spamming", 
      "PvE", "PvP", "afk_group",
      "MyGroup", "Group", "group1",
      "GroupOne", "Stalkers", "my_group"
    ];
    
    const groupNameExample = groupNameExamples[Math.floor(Math.random() * groupNameExamples.length)];

    const card = document.createElement("div");
    card.className = "bot-card";
    card.id = `bot-card-${username}`;
    card.setAttribute("wrapper", "bot-card");
    
    card.innerHTML = `
      <div class="head">
        <div class="top">
          <img src="${steveIconPath.src}" class="image" draggable="false">
          <div class="text">
            <div class="username">${username}</div>
            <div class="status" id="monitoring-status-${username}">waiting</div>
          </div>
        </div>

        <div class="bottom">
          <input type="text" class="bot-group" id="bot-group-${username}" placeholder="Группа, например, ${groupNameExample}">
        </div>
      </div>

      <div class="info">
        <p class="line">Пинг: <span id="monitoring-ping-${username}">${profile.ping}</span>ms</p>
        <p class="line">Здоровье: <span id="monitoring-health-${username}">${profile.health}</span> / 20</p>
        <p class="line">Прокси: <span id="monitoring-proxy-${username}">${profile.proxy.ip_address}</span></p>
        <p class="line">Пароль: <span>${profile.password ?? "No password"}</span></p>
      </div>

      <div class="buttons">
        <button class="min" id="open-chat-${username}">Открыть чат</button>
        <button class="min" id="reset-${username}">Сбросить</button>
        <button class="min" id="disconnect-${username}">Отключить</button>
      </div>
    `;

    this.cards?.appendChild(card);
    this.initializeBotCard(username);
  }

  /** Метод создания обёркти чата у бота */
  private createChatWrapper(username: string): HTMLDivElement {
    const wrapper = document.createElement("div");
    wrapper.className = "cover";
    wrapper.id = `chat-${username}`;
    wrapper.setAttribute("wrapper", "bot-chat");

    wrapper.innerHTML = `
      <div class="panel">
        <div class="left">
          <div class="custom-select">
            <select id="select-chat-filter-${username}">
              <option value="all">Все сообщения</option>
              <option value="bans">Блокировки</option>
              <option value="mentions">Упоминания</option>
              <option value="warnings">Предупреждения</option>
              <option value="links">Ссылки</option>
            </select>
          </div>

          <button class="min" id="filter-chat-${username}">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-filter">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M4 4h16v2.172a2 2 0 0 1 -.586 1.414l-4.414 4.414v7l-6 2v-8.5l-4.48 -4.928a2 2 0 0 1 -.52 -1.345v-2.227" />
            </svg>
          </button>
        </div>

        <div class="right">
          <button class="min" id="clear-chat-${username}">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-trash">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M4 7l16 0" />
              <path d="M10 11l0 6" />
              <path d="M14 11l0 6" />
              <path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" />
              <path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" />
            </svg>
          </button>

          <button class="min" id="close-chat-${username}">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M18 6l-12 12" />
              <path d="M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <div class="chat-content" id="monitoring-chat-content-${username}"></div>

      <div class="pretty-input-wrapper">
        <p class="signature">${username}</p>
        <input type="text" id="chat-message-${username}" placeholder="Введите и отправьте сообщение, нажав «Enter»">
      </div>
    `;

    this.wrappers?.appendChild(wrapper);

    return wrapper;
  }

  /** Метод обновления карточки бота */
  private updateBotCard(username: string, profile: BotProfile): void {
    const status = document.getElementById(`monitoring-status-${username}`) as HTMLSpanElement;
    const proxy = document.getElementById(`monitoring-proxy-${username}`) as HTMLSpanElement;
    const ping = document.getElementById(`monitoring-ping-${username}`) as HTMLSpanElement;
    const health = document.getElementById(`monitoring-health-${username}`) as HTMLSpanElement;
    const group = document.getElementById(`bot-group-${username}`) as HTMLInputElement;

    if (health.innerText.split("/")[0].replace(" ", "") != profile.health.toString()) {
      const card = document.getElementById(`bot-card-${username}`);
      card?.classList.add("glow");
      setTimeout(() => card?.classList.remove("glow"), 300);
    }

    let statusColor = "var(--bot-waiting-status-color)";

    switch (profile.status) {
      case "Connected":
        statusColor = "var(--bot-connected-status-color)";
        break;
      case "Disconnected":
        statusColor = "var(--bot-disconnected-status-color)";
        break;
      case "Waiting":
        statusColor = "var(--bot-waiting-status-color)";
        break;
    }

    status.innerText = String(profile.status).toLowerCase();
    status.style.color = statusColor;
    proxy.innerText = profile.proxy.ip_address ? profile.proxy.ip_address : "No proxy";
    ping.innerText = profile.ping.toString();
    health.innerText = profile.health.toString();

    // Если группа по умолчанию её можно и не отображать, это ни на что не повлияет
    if (profile.group !== "global") group.value = profile.group;
  }

  /** Метод инициализации карточки бота */
  private initializeBotCard(username: string): void {
    this.chatHistoryFilters[username] = "all";
    this.usernameList.push(username);

    const chatWrapper = this.createChatWrapper(username);

    document.getElementById(`bot-group-${username}`)?.addEventListener("input", async () => {
      try {
        const group = (document.getElementById(`bot-group-${username}`) as HTMLInputElement).value.replace(" ", "");

        await invoke("set_group", {
          username: username,
          group: group !== "" ? group : "global"
        });
      } catch (error) {
        logger.log(`Ошибка изменения группы ${username}: ${error}`, "error");
      }
    });

    document.getElementById(`open-chat-${username}`)?.addEventListener("click", () => chatWrapper.style.display = "flex");
    document.getElementById(`close-chat-${username}`)?.addEventListener("click", () => chatWrapper.style.display = "none");

    document.getElementById(`disconnect-${username}`)?.addEventListener("click", async () => {
      try {
        await invoke("quick_operation", {
          operation: "disconnect-bot",
          options: {
            username: username
          }
        });
      } catch (error) {
        logger.log(`Ошибка отключения бота ${username}: ${error}`, "error");
      }
    });

    document.getElementById(`reset-${username}`)?.addEventListener("click", async () => {
      try {
        await invoke("quick_operation", {
          operation: "reset-bot",
          options: {
            username: username
          }
        });
      } catch (error) {
        logger.log(`Ошибка сбрасывания задач и состояний бота ${username}: ${error}`, "error");
      }
    });

    document.getElementById(`filter-chat-${username}`)?.addEventListener("click", () => {
      try {
        const content = document.getElementById(`monitoring-chat-content-${username}`);
        const type = document.getElementById(`select-chat-filter-${username}`) as HTMLSelectElement;
        const history = [...document.querySelectorAll(`[monitoring-message="${username}"]`).values()];
        
        content!.innerHTML = "";
        this.chatHistoryFilters[username] = type.value;

        history.forEach(m => this.filterMessage(type.value, m.textContent || "") ? content?.appendChild(m) : null);
      } catch (error) {
        logger.log(`Ошибка фильтровки чата: ${error}`, "error");
      }
    });

    document.getElementById(`clear-chat-${username}`)?.addEventListener("click", () => {
      const messages = document.querySelectorAll(`[monitoring-message="${username}"]`);
      messages.forEach(msg => msg.remove());
      this.chatMessageCounter[username] = 0;
    });

    document.getElementById(`chat-${username}`)?.addEventListener("keydown", async (e) => {
      if ((e as KeyboardEvent).key === "Enter") {
        const input = document.getElementById(`chat-message-${username}`) as HTMLInputElement;

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

  /** Метод создания триграмм из слова */
  private createTrigrams(word: string): string[] {
    const trigrams = [];
    for (let i = 0; i <= word.length - 3; i++) trigrams.push(word.substring(i, 3));
    return trigrams;
  }

  /** Метод проверки слова на наличие определённых паттернов */
  private checkPatterns(word: string, patterns: string[]): boolean {
    if (word.length < 3) return false;

    let totalTrigrams = 0;
    let similarTrigrams = 0;

    const wts = this.createTrigrams(word);
    totalTrigrams = wts.length;

    for (const p of patterns) for (const wt of wts) for (const pt of this.createTrigrams(p)) wt.toLowerCase() == pt.toLowerCase() ? similarTrigrams++ : null;

    if (similarTrigrams >= totalTrigrams / 2) return true;

    return false;
  }

  /** Метод фильтровки сообщения */
  private filterMessage(type: string, message: string): boolean {
    if (type === "all") {
      return true;
    } else if (type === "links") {
      const patterns = [
        "http://", "https://", ".dev", ".com", ".org",
        ".io", ".ai", ".net", ".pro", ".gov", ".lv",
        ".ru", ".onion", ".ie", ".co", ".fun", ".gg",
        ".xyz", ".club", ".eu", ".me", ".us", ".online",
        ".br", ".cc", ".no"
      ];

      let result = false;

      for (const p of patterns) {
        if (message.includes(p)) {
          result = true;
          break;
        }
      }
      
      return result;
    } else {
      const patterns: Record<string, string[]> = {
        bans: [
          "banned", "ban", "kicked",
          "kick", "кикнут", "заблокированный",
          "заблокирован", "блокировка",
          "заблокировали", "забанен", 
          "забанили", "бан", "blocked"
        ],
        warnings: [
          "предупреждение", "warn", "warning", 
          "важно", "important", "предупреждает", 
          "важная", "уведомление", "осведомление",
          "notice", "уведомлять", "замечание"
        ],
        mentions: [
          "упомянут", "mention", "reference", 
          "упоминает", "упоминание", "ссылаться"
        ]
      };

      let results: boolean[] = [];
      for (const word of message.split(" ")) results.push(this.checkPatterns(word, patterns[type]));
      if (results.includes(true)) return true;
    }
      
    return false;
  }
}   

const monitoringModule = new MonitoringModule();

export { monitoringModule }
