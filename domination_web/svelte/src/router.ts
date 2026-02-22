import type { Component } from "svelte";

type RouteMap = Record<string, () => Component>;

let currentPath = window.location.pathname;
let routes: RouteMap = {};

function render(): void {
  currentPath = window.location.pathname;
}

export function initRouter(map: RouteMap): void {
  routes = map;
  window.addEventListener("popstate", render);
  render();
}

export function navigate(path: string): void {
  history.pushState({}, "", path);
  render();
}

export function redirect(path: string): void {
  history.replaceState({}, "", path);
  render();
}

export function getComponent(): Component {
  return routes[currentPath]();
}

