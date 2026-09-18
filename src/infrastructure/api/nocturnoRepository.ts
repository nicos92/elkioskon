import { invoke } from "@tauri-apps/api/core";
import type { NocturnoConfig } from "../../domain/entities";
import type { INocturnoConfigRepository } from "../../domain/interfaces";
import { getCurrentUserId } from "../utils/currentUser";

export class NocturnoConfigApiRepository implements INocturnoConfigRepository {

  async getConfig(): Promise<NocturnoConfig> {
    return await invoke<NocturnoConfig>("get_nocturno_config", {
      userId: getCurrentUserId(),
    });
  }

  async saveConfig(config: NocturnoConfig): Promise<void> {
    return await invoke<void>("save_nocturno_config", {
      userId: getCurrentUserId(),
      request: {
        activo: config.activo,
        hora_inicio: config.hora_inicio,
        hora_fin: config.hora_fin,
      },
    });
  }
}