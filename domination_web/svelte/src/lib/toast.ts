import { writable } from "svelte/store";

export type ToastType = "info" | "warn" | "error";

export type Toast = {
  id: number;
  message: string;
  type: ToastType;
  duration: number;
};

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);
  let counter = 0;

  function remove(id: number) {
    update((toasts) => toasts.filter((t) => t.id !== id));
  }

  function notify(
    message: string,
    type: ToastType = "info",
    duration = 3000
  ) {
    const id = ++counter;

    update((toasts) => [
      ...toasts,
      { id, message, type, duration },
    ]);

    if (duration > 0) {
      setTimeout(() => remove(id), duration);
    }
  }

  return {
    subscribe,
    notify,
    remove,
  };
}

export const toast = createToastStore();
