import type { JsonValue } from "./jsonTools";

interface XmlElement {
  readonly name: string;
  readonly attributes?: Readonly<Record<string, string>>;
}

const XML_NAME_PATTERN = /^[A-Za-z_][A-Za-z0-9_.-]*$/;
const TYPESCRIPT_PROPERTY_PATTERN = /^[$A-Z_a-z][$\w]*$/;
const INDENT_SIZE = 2;

export function jsonToXml(value: JsonValue) {
  const declaration = '<?xml version="1.0" encoding="UTF-8"?>';
  return [declaration, ...renderXmlElement({ name: "root" }, value, 0)].join("\n");
}

export function jsonToTypeScript(value: JsonValue) {
  return `export type Root = ${inferTypeScriptType(value, 0)};\n`;
}

function renderXmlElement(element: XmlElement, value: JsonValue, depth: number): string[] {
  const indentation = indent(depth);
  const openingTag = `<${element.name}${serializeAttributes(element.attributes)}>`;
  if (isScalar(value)) {
    return [`${indentation}${openingTag}${escapeXml(String(value))}</${element.name}>`];
  }
  const children = isJsonArray(value)
    ? value.flatMap((item) => renderXmlElement({ name: "item" }, item, depth + 1))
    : Object.entries(value).flatMap(([key, item]) => renderXmlProperty(key, item, depth + 1));
  if (children.length === 0) {
    return [`${indentation}<${element.name}${serializeAttributes(element.attributes)} />`];
  }
  return [`${indentation}${openingTag}`, ...children, `${indentation}</${element.name}>`];
}

function renderXmlProperty(key: string, value: JsonValue, depth: number) {
  const element = XML_NAME_PATTERN.test(key)
    ? { name: key }
    : { name: "property", attributes: { name: key } };
  return renderXmlElement(element, value, depth);
}

function serializeAttributes(attributes?: Readonly<Record<string, string>>) {
  if (!attributes) {
    return "";
  }
  return Object.entries(attributes)
    .map(([name, value]) => ` ${name}="${escapeXml(value)}"`)
    .join("");
}

function escapeXml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

function isScalar(value: JsonValue): value is null | boolean | number | string {
  return value === null || typeof value !== "object";
}

function isJsonArray(value: JsonValue): value is readonly JsonValue[] {
  return Array.isArray(value);
}

function inferTypeScriptType(value: JsonValue, depth: number): string {
  if (value === null) return "null";
  if (isJsonArray(value)) return inferArrayType(value, depth);
  if (typeof value === "object") return inferObjectType(value, depth);
  return typeof value;
}

function inferArrayType(values: readonly JsonValue[], depth: number) {
  if (values.length === 0) {
    return "unknown[]";
  }
  const itemTypes = [...new Set(values.map((value) => inferTypeScriptType(value, depth)))];
  const itemType = itemTypes.length === 1 ? itemTypes[0] : `(${itemTypes.join(" | ")})`;
  return `${itemType}[]`;
}

function inferObjectType(value: Readonly<Record<string, JsonValue>>, depth: number) {
  const entries = Object.entries(value);
  if (entries.length === 0) {
    return "Record<string, never>";
  }
  const properties = entries.map(([key, item]) => {
    const propertyName = TYPESCRIPT_PROPERTY_PATTERN.test(key) ? key : JSON.stringify(key);
    return `${indent(depth + 1)}${propertyName}: ${inferTypeScriptType(item, depth + 1)};`;
  });
  return `{\n${properties.join("\n")}\n${indent(depth)}}`;
}

function indent(depth: number) {
  return " ".repeat(depth * INDENT_SIZE);
}
