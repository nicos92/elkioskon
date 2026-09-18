import { ref } from "vue";

export interface Toast {
  id: number;
  type: "error" | "success" | "warning";
  message: string;
}

const toasts = ref<Toast[]>([]);
let nextId = 1;

const AUTO_DISMISS_MS = 4000;

function push(type: Toast["type"], message: string) {
  const id = nextId++;
  toasts.value.push({ id, type, message });
  setTimeout(() => {
    remove(id);
  }, AUTO_DISMISS_MS);
}

function remove(id: number) {
  const index = toasts.value.findIndex((t) => t.id === id);
  if (index !== -1) {
    toasts.value.splice(index, 1);
  }
}

export function useToasts() {
  function error(message: string) {
    push("error", message);
  }

  function success(message: string) {
    push("success", message);
  }

  function warning(message: string) {
    push("warning", message);
  }

  return { toasts, error, success, warning };
}
