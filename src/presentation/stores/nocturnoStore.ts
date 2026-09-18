import { defineStore } from "pinia";
import { ref } from "vue";
import type { NocturnoConfig } from "../../domain/entities";
import { toErrorMessage } from "../../infrastructure/api/errorHandler";
import { nocturnoConfigRepository } from "../../infrastructure/di";
import { NocturnoUseCase } from "../../application/usecases";

export const useNocturnoStore = defineStore("nocturno", () => {
  const nocturnoUseCase = new NocturnoUseCase(nocturnoConfigRepository);
  const config = ref<NocturnoConfig>({
    activo: false,
    hora_inicio: "22:00",
    hora_fin: "06:00",
  });
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  async function fetchConfig() {
    isLoading.value = true;
    error.value = null;
    try {
      config.value = await nocturnoUseCase.getConfig();
    } catch (e) {
      error.value = toErrorMessage(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function saveConfig(cfg: NocturnoConfig): Promise<boolean> {
    isLoading.value = true;
    error.value = null;
    try {
      await nocturnoUseCase.saveConfig(cfg);
      config.value = { ...cfg };
      return true;
    } catch (e) {
      error.value = toErrorMessage(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  return {
    config,
    isLoading,
    error,
    fetchConfig,
    saveConfig,
  };
});