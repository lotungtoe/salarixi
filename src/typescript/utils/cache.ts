import { invoke } from "@tauri-apps/api/core";
import { date } from "./date";

/** Тип времени */
export enum TimeType {
  Day,
  Hours,
}

/** Функция проверки даты, истекла та или нет */
function isDataExpired(
  saveDateString: string,
  maxStorageValue: number,
  timeType: TimeType
): boolean {
  const dateRegex = /^(\d{1,2})\.(\d{1,2})\.(\d{4})-(\d{1,2}):(\d{1,2}):(\d{1,2})$/;
  const match = saveDateString.match(dateRegex);
  if (!match) return false;

  const [, day, month, year, hours, minutes, seconds] = match.map(Number);
  const saveDate = new Date(year, month - 1, day, hours, minutes, seconds);
  if (isNaN(saveDate.getTime())) return false;

  const nowDate = new Date();
  const expirationDate = new Date(saveDate);

  if (timeType === TimeType.Day) {
    expirationDate.setDate(expirationDate.getDate() + maxStorageValue);
  } else if (timeType === TimeType.Hours) {
    expirationDate.setHours(expirationDate.getHours() + maxStorageValue);
  } else {
    return false;
  }

  return nowDate > expirationDate;
}

/** Функция сохранения кэша */
export async function saveCache(cache: string, filename: string): Promise<CommandResult<null>> {
  const encoder = new TextEncoder();
  const uint8 = encoder.encode(cache);

  return await invoke("save_cache", {
    cache: Array.from(uint8),
    filename: filename,
    date: `${date("exact")}-${date("common")}`,
  });
}

/** Функция загрузки кэша */
export async function loadCache(name: string, filename: string, maxStorageValue: number, timeType: TimeType): Promise<CommandResult<string>> {
  const result1 = await invoke("get_cache_date", { name: name }) as CommandResult<string>;
  if (!result1.data) {
    return {
      data: null,
      error: result1.error,
    };
  }

  const result2 = await invoke("load_cache", { filename: filename }) as CommandResult<number[]>;
  if (!result2.data) {
    return {
      data: null,
      error: result2.error,
    };
  }

  const uint8 = new Uint8Array(result2.data);
  const decoder = new TextDecoder("utf-8");
  const cacheStr = decoder.decode(uint8);

  return {
    data: cacheStr,
    error: isDataExpired(result1.data, maxStorageValue, timeType) ? "expired" : null,
  };
}
