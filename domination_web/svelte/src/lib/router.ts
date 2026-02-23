import type { Component } from "svelte";
import { writable } from "svelte/store";

type RouteMap = Record<string, () => Promise<Component>>;

const currentPath = writable(getHashPath());
let routes: RouteMap = {};

function getHashPath(): string {
  const hash = window.location.hash.replace(/^#/, "");
  return hash || "/";
}

export function initRouter(map: RouteMap): void {
  if (!window.location.hash) {
    window.location.hash = "/";
  }

  routes = map;

  window.addEventListener("hashchange", () => {
    currentPath.set(getHashPath());
  });

  currentPath.set(getHashPath());
}

export function navigate(path: string): void {
  window.location.hash = path;
}

export function redirect(path: string): void {
  const url = `${window.location.pathname}#${path}`;
  history.replaceState({}, "", url);
  currentPath.set(path);
}

export async function getComponent(path: string): Promise<Component> {
  const route = routes[path];
  if (!route) throw new Error("Route not found");
  return route();
}

export { currentPath };
