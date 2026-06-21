import { open } from "@tauri-apps/plugin-dialog";

import { date } from "../utils/date";
import { logger } from "../utils/logger";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import { join } from "@tauri-apps/api/path";
import { messages } from "../utils/message";

/** Структура данных аккаунта */
interface Account {
  creation_date: string;
  password: string | null;
  email: string | null;
  proxy: string | null;
  proxy_username: string | null;
  proxy_password: string | null;
  selected: boolean;
}

/** Модуль управления аккаунтами */
class AccountModule {
  private accountList: HTMLDivElement | null = null;
  private accountCounter: HTMLSpanElement | null = null;
  private wrappers: HTMLDivElement | null = null;

  private accounts: Record<string, Account | null> = {};

  private noSelected: boolean = true;

  /** Метод инициализации функций, связанных с аккаунтами */
  public init(): void {
    this.accountList = document.getElementById("account-list") as HTMLDivElement;
    this.accountCounter = document.getElementById("account-counter") as HTMLSpanElement;
    this.wrappers = document.getElementById("account-wrappers") as HTMLDivElement;

    this.loadSavedAccounts();

    const accountsContainer = document.getElementById("accounts-container") as HTMLDivElement;

    const selectedAccountsEditor = document.getElementById("selected-accounts-editor") as HTMLDivElement;
    const selectedAccountsPassword = document.getElementById("selected-accounts-password") as HTMLInputElement;
    const selectedAccountsEmail = document.getElementById("selected-accounts-email") as HTMLInputElement;
    const selectedAccountsProxy = document.getElementById("selected-accounts-proxy") as HTMLInputElement;
    const selectedAccountsProxyUsername = document.getElementById("selected-accounts-proxy-username") as HTMLInputElement;
    const selectedAccountsProxyPassword = document.getElementById("selected-accounts-proxy-password") as HTMLInputElement;

    const addAccountBtn = document.getElementById("add-account") as HTMLButtonElement;
    const openSelectedAccountsEditorBtn = document.getElementById("edit-selected-accounts") as HTMLButtonElement;
    const closeSelectedAccountsEditorBtn = document.getElementById("close-selected-accounts-editor") as HTMLButtonElement;
    const removeSelectedAccountsBtn = document.getElementById("remove-selected-accounts") as HTMLButtonElement;
    const importAccounts = document.getElementById("import-accounts") as HTMLButtonElement;
    const exportAccounts = document.getElementById("export-accounts") as HTMLButtonElement;

    accountsContainer.addEventListener("keydown", e => {
      if (e.key.toLowerCase() !== "escape") return;

      let count = 0;

      for (const username in this.accounts) {
        const account = this.accounts[username];
        if (!account || (this.noSelected ? account.selected : !account.selected)) continue;

        account.selected = this.noSelected;
        (document.getElementById(`account-${username}-selector-flag`) as HTMLInputElement).checked = this.noSelected;

        count++;
      }

      if (count > 0) {
        this.noSelected = !this.noSelected;
        this.changeSelectorsState();
      }
    });

    addAccountBtn.addEventListener("click", () => {
      const usernameInput = document.getElementById("account-username") as HTMLInputElement;
      const username = usernameInput.value;

      if (this.accounts[username] || username === "") return;

      usernameInput.value = "";

      this.createAccountCard(username, {
        creation_date: date("exact"),
        password: null,
        email: null,
        proxy: null,
        proxy_username: null,
        proxy_password: null,
        selected: false,
      });

      this.updateAccountCounter();

      logger.log(`Создан новый аккаунт "${username}"`, "system");
    });

    openSelectedAccountsEditorBtn.addEventListener("click", () => selectedAccountsEditor.style.display = "flex");
    closeSelectedAccountsEditorBtn.addEventListener("click", () => selectedAccountsEditor.style.display = "none");

    selectedAccountsPassword.addEventListener("input", () => this.setValueForAllSelected("password", selectedAccountsPassword.value));
    selectedAccountsEmail.addEventListener("input", () => this.setValueForAllSelected("email", selectedAccountsEmail.value));
    selectedAccountsProxy.addEventListener("input", () => this.setValueForAllSelected("proxy", selectedAccountsProxy.value));
    selectedAccountsProxyUsername.addEventListener("input", () => this.setValueForAllSelected("proxy_username", selectedAccountsProxyUsername.value));
    selectedAccountsProxyPassword.addEventListener("input", () => this.setValueForAllSelected("proxy_password", selectedAccountsProxyPassword.value));

    removeSelectedAccountsBtn.addEventListener("click", () => {
      let removedAccounts = [];

      for (const username in this.accounts) {
        const account = this.accounts[username];
        if (!account || !account.selected) continue;

        delete this.accounts[username];

        document.getElementById(`account-editor-${username}`)?.remove();
        document.getElementById(`account-card-${username}`)?.remove();

        removedAccounts.push(username);
      }

      this.noSelected = true;
      this.changeSelectorsState();

      messages.message("Аккаунты", `Удалено ${removedAccounts.length} аккаунтов`);
      logger.log(`Аккаунты ${removedAccounts.join(", ")} удалены`, "system");
    });

    importAccounts.addEventListener("click", async () => {
      try {
        const path = await open({
          directory: false,
          multiple: false,
          filters: [{
            name: "Accounts",
            extensions: ["json"]
          }]
        });

        if (!path) return;

        this.clearAccounts();

        const buffer = await readFile(path);

        const decoder = new TextDecoder();
        const accounts = JSON.parse(decoder.decode(buffer));

        for (const username in accounts) {
          const data = accounts[username];
          if (!data) continue;

          this.createAccountCard(username, {
            creation_date: date("exact"),
            password: data["password"],
            email: data["email"],
            proxy: data["proxy"],
            proxy_username: data["proxy_username"] ?? data["proxy-username"],
            proxy_password: data["proxy_password"] ?? data["proxy-password"],
            selected: false,
          });
        } 
        
        this.updateAccountCounter();

        messages.message("Импорт аккаунтов", `Аккаунты успешно импортированы`);
      } catch (error) {
        logger.log(`Ошибка импорта аккаунтов: ${error}`, "error");
      }
    });

    exportAccounts.addEventListener("click", async () => {
      try {
        const directory = await open({
          directory: true,
          multiple: false
        });

        if (!directory) return;

        const accounts: any = {};

        for (const username in this.accounts) {
          const data = this.accounts[username];
          if (!data) continue;

          accounts[username] = {
            password: data.password,
            email: data.email,
            proxy: data.proxy,
            proxy_username: data.proxy_username,
            proxy_password: data.proxy_password
          };
        }

        const path = await join(directory, "salarixi.accounts.json");
        let encoder = new TextEncoder();
        let buffer = encoder.encode(JSON.stringify(accounts, null, 2));

        await writeFile(path, buffer);

        messages.message("Экспорт аккаунтов", `Аккаунты успешно экспортированы`);
      } catch (error) {
        logger.log(`Ошибка экспорта аккаунтов: ${error}`, "error");
      }
    });

    setInterval(() => {
      let cleanAccounts: any = {};

      for (const username in this.accounts) {
        const account = this.accounts[username];
        if (!account) continue;

        cleanAccounts[username] = {
          creation_date: account.creation_date,
          password: account.password,
          email: account.email,
          proxy: account.proxy,
          proxy_username: account.proxy_username,
          proxy_password: account.proxy_password,
        };
      }

      localStorage.setItem("accounts", JSON.stringify(cleanAccounts))
    }, 2000);
  }

  /** Метод получения всех существующих аккаунтов (без лишних полей) */
  public getAccounts(): Record<string, any> {
    let cleanAccounts: any = {};

    for (const username in this.accounts) {
      const account = this.accounts[username];
      if (!account) continue;

      cleanAccounts[username] = {
        password: account.password,
        email: account.email,
        proxy: account.proxy,
        proxy_username: account.proxy_username,
        proxy_password: account.proxy_password,
      };
    }

    return cleanAccounts;
  }

  /** Метод обновления счётчика аккаунтов */
  private updateAccountCounter(): void {
    let count = 0;
    for (const _ in this.accounts) count++;
    if (this.accountCounter) this.accountCounter.innerText = count.toString();
  }

  /** Метод измнения состояния кнопок, отвечающих за взаимодействие с выбранными аккаунтами */
  private changeSelectorsState(): void {
    (document.getElementById("edit-selected-accounts") as HTMLButtonElement).disabled = this.noSelected;
    (document.getElementById("remove-selected-accounts") as HTMLButtonElement).disabled = this.noSelected;
  }

  /** Метод измнения значения всех выбранных аккаунтов */
  private setValueForAllSelected(
    field: "password" | "email" | "proxy" | "proxy_username" | "proxy_password",
    value: string,
  ): void {
    for (const username in this.accounts) {
      const account = this.accounts[username];
      if (!account || !account.selected) continue;

      account[field] = value;

      const input = document.getElementById(`account-${username}-${field}`) as HTMLInputElement | null;
      if (input) input.value = value;
    }
  }

  /** Метод полной очистки всех аккаунтов */
  private clearAccounts(): void {
    document.querySelectorAll('[wrapper="account-editor"]').forEach(w => w.remove());
    document.querySelectorAll('[wrapper="account-card"]').forEach(w => w.remove());

    this.accounts = {};
    localStorage.removeItem("accounts");

    this.updateAccountCounter();
  }

  /** Метод загрузки сохранённых аккаунтов из локального хранилища */
  private loadSavedAccounts(): void {
    const accounts_json = localStorage.getItem("accounts");
    if (!accounts_json) return;

    const accounts = Object.entries<Account | null>(JSON.parse(accounts_json));
    if (accounts.length < 1) return;

    for (const [username, fields] of accounts) fields ? this.createAccountCard(username, fields) : null;

    this.updateAccountCounter();

    logger.log(`Сохранённые аккаунты загружены (${accounts.length} штук)`, "system");
  }

  /** Метод создания карточки аккаунта */
  private createAccountCard(username: string, account: Account): void {
    this.accounts[username] = account;

    const editorWrapper = this.createEditWrapper(username, account);

    const row = document.createElement("div");
    row.className = "account-row";
    row.id = `account-card-${username}`;
    row.setAttribute("wrapper", "account-card");

    const usernameCell = document.createElement("div");
    usernameCell.className = "account-cell username-cell";

    const chbxContainer = document.createElement("label");
    chbxContainer.className = "checkbox-container";

    const accountSelectorFlag = document.createElement("input");
    accountSelectorFlag.id = `account-${username}-selector-flag`;
    accountSelectorFlag.type = "checkbox";

    accountSelectorFlag.addEventListener("change", () => {
      const account = this.accounts[username];
      if (!account) return;

      account.selected = accountSelectorFlag.checked;

      if (accountSelectorFlag.checked) {
        if (this.noSelected) this.noSelected = false;
      } else {
        let noSelected = true;

        for (const u in this.accounts) {
          const a = this.accounts[u];
          if (!a) continue;

          if (a.selected) {
            noSelected = false;
            break;
          }
        }

        this.noSelected = noSelected;
      }

      this.changeSelectorsState();
    });

    const accountSelectorMark = document.createElement("span");
    accountSelectorMark.className = "checkmark";

    const usernameText = document.createElement("span");
    usernameText.innerText = username;

    chbxContainer.appendChild(accountSelectorFlag);
    chbxContainer.appendChild(accountSelectorMark);
    usernameCell.appendChild(chbxContainer);
    usernameCell.appendChild(usernameText);

    const creationDateCell = document.createElement("div");
    creationDateCell.className = "account-cell date-cell";
    creationDateCell.innerText = account.creation_date;

    const actionsCell = document.createElement("div");
    actionsCell.className = "account-cell actions-cell";

    const editAccountBtn = document.createElement("button");
    editAccountBtn.className = "action-btn";
    editAccountBtn.addEventListener("click", () => editorWrapper.style.display = "flex");
    editAccountBtn.innerHTML = `
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
        <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492M5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0"/>
        <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.42 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.42-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.42-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.115z"/>
      </svg>
    `;

    const removeAccountBtn = document.createElement("button");
    removeAccountBtn.className = "action-btn";

    removeAccountBtn.addEventListener("click", () => {
      row.remove();
      editorWrapper.remove();
      delete this.accounts[username];
      this.updateAccountCounter();
    });

    removeAccountBtn.innerHTML = `
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
        <path d="M2.5 1a1 1 0 0 0-1 1v1a1 1 0 0 0 1 1H3v9a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2V4h.5a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1H10a1 1 0 0 0-1-1H7a1 1 0 0 0-1 1zm3 4a.5.5 0 0 1 .5.5v7a.5.5 0 0 1-1 0v-7a.5.5 0 0 1 .5-.5M8 5a.5.5 0 0 1 .5.5v7a.5.5 0 0 1-1 0v-7A.5.5 0 0 1 8 5m3 .5v7a.5.5 0 0 1-1 0v-7a.5.5 0 0 1 1 0"/>
      </svg>
    `;

    actionsCell.appendChild(editAccountBtn);
    actionsCell.appendChild(removeAccountBtn);

    row.appendChild(usernameCell);
    row.appendChild(creationDateCell);
    row.appendChild(actionsCell);

    this.accountList?.appendChild(row);
  }

  /** Метод создания обёркти редактора настроек аккаунта */
  private createEditWrapper(username: string, account: any): HTMLDivElement {
    const wrapper = document.createElement("div");
    wrapper.className = "cover";
    wrapper.id = `account-editor-${username}`;
    wrapper.setAttribute("wrapper", "account-editor");

    const coverPanel = document.createElement("div");
    coverPanel.className = "panel with-header";
    coverPanel.style.marginBottom = "20px";

    const coverPanelLeft = document.createElement("div");
    coverPanelLeft.className = "left";
    coverPanelLeft.innerHTML = `<div class="header">Редактирование аккаунта "${username}"</div>`;

    const coverPanelRight = document.createElement("div");
    coverPanelRight.className = "right";

    const closeEditorBtn = document.createElement("button");
    closeEditorBtn.className = "btn min";
    closeEditorBtn.addEventListener("click", () => wrapper.style.display = "none");
    closeEditorBtn.innerHTML = `
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
        <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
      </svg>
    `;

    coverPanelRight.appendChild(closeEditorBtn);

    coverPanel.appendChild(coverPanelLeft);
    coverPanel.appendChild(coverPanelRight);

    const settingsContainer = document.createElement("div");
    settingsContainer.className = "account-settings";

    const options = [
      {
        name: "Пароль",
        tag: "password",
        placeholder: "qwerty12345",
      },
      {
        name: "Эл. почта",
        tag: "email",
        placeholder: "qwerty@gmail.com",
      },
      {
        name: "Прокси SOCKS5",
        tag: "proxy",
        placeholder: "socks5://35.91.83.91:1111",
      },
      {
        name: "Юзернейм прокси",
        tag: "proxy-username",
        placeholder: "user",
      },
      {
        name: "Пароль прокси",
        tag: "proxy-password",
        placeholder: "root",
      },
    ];

    for (const option of options) {
      const e = document.createElement("div");
      e.className = "pretty-input-wrapper";

      const signature = document.createElement("p");
      signature.className = "signature fix";
      signature.innerText = option.name;

      const input = document.createElement("input");
      input.id = `account-${username}-${option.tag}`;
      input.type = "text";
      input.placeholder = option.placeholder;

      const fieldName = option.tag.replaceAll("-", "_");
      account[fieldName] ? input.value = account[fieldName] : null;
      input.addEventListener("input", () => account[fieldName] = input.value);

      e.appendChild(signature);
      e.appendChild(input);

      settingsContainer.appendChild(e);
    }

    wrapper.appendChild(coverPanel);
    wrapper.appendChild(settingsContainer);

    this.wrappers?.appendChild(wrapper);

    return wrapper;
  }
}

const accountModule = new AccountModule();

export { accountModule }
