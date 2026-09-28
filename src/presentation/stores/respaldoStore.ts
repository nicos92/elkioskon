import { defineStore } from "pinia";
import { ref } from "vue";
import type {
  RespaldoInfo,
  RespaldoResult,
  RestauracionResult,
} from "../../domain/entities";
import { toErrorMessage } from "../../infrastructure/api/errorHandler";
import { respaldoRepository } from "../../infrastructure/di";
import { RespaldoUseCase } from "../../application/usecases";

export const useRespaldoStore = defineStore("respaldo", () => {
  const respaldoUseCase = new RespaldoUseCase(respaldoRepository);
  const info = ref<RespaldoInfo | null>(null);
  const ultimoRespaldo = ref<RespaldoResult | null>(null);
  const ultimaRestauracion = ref<RestauracionResult | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  async function fetchInfo() {
    isLoading.value = true;
    error.value = null;
    try {
      info.value = await respaldoUseCase.getInfo();
    } catch (e) {
      error.value = toErrorMessage(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function crearRespaldo(destino: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;
    try {
      ultimoRespaldo.value = await respaldoUseCase.crearRespaldo(destino);
      return true;
    } catch (e) {
      error.value = toErrorMessage(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  async function restaurarRespaldo(origen: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;
    try {
      ultimaRestauracion.value =
        await respaldoUseCase.restaurarRespaldo(origen);
      return true;
    } catch (e) {
      error.value = toErrorMessage(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  return {
    info,
    ultimoRespaldo,
    ultimaRestauracion,
    isLoading,
    error,
    fetchInfo,
    crearRespaldo,
    restaurarRespaldo,
  };
});
