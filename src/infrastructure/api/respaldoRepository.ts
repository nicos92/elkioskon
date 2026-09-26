import { invoke } from "@tauri-apps/api/core";
import type { RespaldoInfo, RespaldoResult } from "../../domain/entities";
import type { IRespaldoRepository } from "../../domain/interfaces";
import { getCurrentUserId } from "../utils/currentUser";

export class RespaldoApiRepository implements IRespaldoRepository {
  async crearRespaldo(destino: string): Promise<RespaldoResult> {
    return await invoke<RespaldoResult>("crear_respaldo", {
      userId: getCurrentUserId(),
      request: { destino },
    });
  }

  async getInfo(): Promise<RespaldoInfo> {
    return await invoke<RespaldoInfo>("get_respaldo_info", {
      userId: getCurrentUserId(),
    });
  }
}
