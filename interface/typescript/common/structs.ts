import { Buffer } from "../utils/buffer";

interface PluginData {
	name: string;
	enabled: boolean;
	modified: string;
	settings: Array<PluginOption>;
}

export interface PluginOption {
	key: string;
	placeholder: string;
	vtype: PluginValueType;
	value: any;
	default: any;
}

export enum PluginValueType {
	U32,
	F32,
	F64,
	Bool,
	String,
}

/** Вспомогательная функция конвертации типа значения опции плагина в тип ввода */
export function pluginValueTypeToInputType(type: PluginValueType): string {
	switch (type) {
		case PluginValueType.U32:
		case PluginValueType.F32:
		case PluginValueType.F64:
			return "number";
		case PluginValueType.Bool:
		case PluginValueType.String:
			return "text";
	}
}

/**
 * Локальная структура, содержащая основные данные плагинов текущей версии программы.
 * 
 * Хранит ID, имя, статус, дату обновления, опции плагина.
 * */
export const plugins: Record<string, PluginData> = {
	"instant-armor-equip": {
		name: "Instant Armor Equip",
		enabled: false,
		modified: "02.09.2026",
		settings: [
			{
				key: "tick-interval",
				placeholder: "100",
				vtype: PluginValueType.U32,
				value: 100,
				default: 100,
			},
			{
				key: "delay-after-no-armor",
				placeholder: "1200",
				vtype: PluginValueType.U32,
				value: 1200,
				default: 1200,
			},
			{
				key: "include-helmet",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
			{
				key: "include-chestplate",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
			{
				key: "include-leggings",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
			{
				key: "include-boots",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
			{
				key: "compare-by-material",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
			{
				key: "compare-by-enchantments",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
			{
				key: "compare-by-durability",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: false,
				default: false,
			},
			{
				key: "adaptation-by-dimension",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
			{
				key: "adaptation-by-state",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: false,
				default: false,
			},
		],
	},
	"auto-totem": {
		name: "Auto Totem",
		enabled: false,
		modified: "02.09.2026",
		settings: [
			{
				key: "tick-interval",
				placeholder: "100",
				vtype: PluginValueType.U32,
				value: 100,
				default: 100,
			},
			{
				key: "delay-after-taking",
				placeholder: "800",
				vtype: PluginValueType.U32,
				value: 800,
				default: 800,
			},
			{
				key: "ignore-item-in-offhand",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
		],
	},
	"auto-eat": {
		name: "Auto Eat",
		enabled: false,
		modified: "01.10.2026",
		settings: [
			{
				key: "tick-interval",
				placeholder: "100",
				vtype: PluginValueType.U32,
				value: 100,
				default: 100,
			},
			{
				key: "minimum-satiety-threshold",
				placeholder: "20",
				vtype: PluginValueType.U32,
				value: 20,
				default: 20,
			},
			{
				key: "minimum-health-threshold",
				placeholder: "12.0",
				vtype: PluginValueType.F32,
				value: 12.0,
				default: 12.0,
			},
			{
				key: "delay-after-no-eat",
				placeholder: "1000",
				vtype: PluginValueType.U32,
				value: 1000,
				default: 1000,
			},
			{
				key: "effects-check",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
		],
	},
	"potion-consumer": {
		name: "Potion Consumer",
		enabled: false,
		modified: "02.09.2026",
		settings: [
			{
				key: "tick-interval",
				placeholder: "200",
				vtype: PluginValueType.U32,
				value: 200,
				default: 200,
			},
			{
				key: "minimum-health-threshold",
				placeholder: "20.0",
				vtype: PluginValueType.F32,
				value: 20.0,
				default: 20.0,
			},
			{
				key: "include-drinking-potion",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
			{
				key: "include-splash-potion",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
			{
				key: "delay-after-no-potion",
				placeholder: "1000",
				vtype: PluginValueType.U32,
				value: 1000,
				default: 1000,
			},
		],
	},
	"auto-look": {
		name: "Auto Look",
		enabled: false,
		modified: "02.09.2026",
		settings: [
			{
				key: "tick-interval",
				placeholder: "100",
				vtype: PluginValueType.U32,
				value: 100,
				default: 100,
			},
			{
				key: "include-entities",
				placeholder: "$player_123, monster, $nullclyze",
				vtype: PluginValueType.String,
				value: "any",
				default: "any",
			},
			{
				key: "distance",
				placeholder: "14.5",
				vtype: PluginValueType.F64,
				value: 14.5,
				default: 14.5,
			},
			{
				key: "with-eye-height",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: true,
				default: true,
			},
		],
	},
	"auto-shield": {
		name: "Auto Shield",
		enabled: false,
		modified: "02.09.2026",
		settings: [
			{
				key: "tick-interval",
				placeholder: "100",
				vtype: PluginValueType.U32,
				value: 100,
				default: 100,
			},
			{
				key: "delay-after-not-equipped",
				placeholder: "1000",
				vtype: PluginValueType.U32,
				value: 1000,
				default: 1000,
			},
			{
				key: "dangerous-entities",
				placeholder: "$player_123, monster, $nullclyze",
				vtype: PluginValueType.String,
				value: "monster, player",
				default: "monster, player",
			},
			{
				key: "distance",
				placeholder: "8.0",
				vtype: PluginValueType.F64,
				value: 8.0,
				default: 8.0,
			},
			{
				key: "ignore-item-in-offhand",
				placeholder: "true | false",
				vtype: PluginValueType.Bool,
				value: false,
				default: false,
			},
		],
	},
	"auto-mending": {
		name: "Auto Mending",
		enabled: false,
		modified: "02.09.2026",
		settings: [
			{
				key: "tick-interval",
				placeholder: "900",
				vtype: PluginValueType.U32,
				value: 900,
				default: 900,
			},
		],
	},
	"pearl-leave": {
		name: "Pearl Leave",
		enabled: false,
		modified: "02.09.2026",
		settings: [
			{
				key: "tick-interval",
				placeholder: "250",
				vtype: PluginValueType.U32,
				value: 250,
				default: 250,
			},
			{
				key: "minimum-health-threshold",
				placeholder: "6.0",
				vtype: PluginValueType.F32,
				value: 6.0,
				default: 6.0,
			},
			{
				key: "maximum-health-threshold",
				placeholder: "10.0",
				vtype: PluginValueType.F32,
				value: 10.0,
				default: 10.0,
			},
			{
				key: "delay-after-no-pearl",
				placeholder: "1000",
				vtype: PluginValueType.U32,
				value: 1000,
				default: 1000,
			},
		],
	},
};

/** Вспомогательная функция записи инормации о плагинах в буффер */
export function writePluginsToBuf(buf: Buffer) {
	const fixedarr = [
		"instant-armor-equip",
		"auto-totem",
		"auto-eat",
		"potion-consumer",
		"auto-look",
		"auto-shield",
		"auto-mending",
		"pearl-leave",
	];

	for (const name of fixedarr) {
		const body = plugins[name];
		const enabled = body.enabled;
		const settings = body.settings;

		buf.writeBoolean(enabled);

		for (const opt of settings) {
			switch (opt["vtype"]) {
				case PluginValueType.U32:
					buf.writeU32(opt["value"]);
					break;
				case PluginValueType.F32:
					buf.writeF32(opt["value"]);
					break;
				case PluginValueType.F64:
					buf.writeF64(opt["value"]);
					break;
				case PluginValueType.Bool:
					buf.writeBoolean(opt["value"]);
					break;
				case PluginValueType.String:
					buf.writeString(opt["value"]);
					break;
			}
		}
	}
}
