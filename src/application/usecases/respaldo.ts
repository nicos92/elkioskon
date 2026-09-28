import type {
  RespaldoInfo,
  RespaldoResult,
  RestauracionResult,
} from "../../domain/entities";
import type { IRespaldoRepository } from "../../domain/interfaces";

export class RespaldoUseCase {
  constructor(private repository: IRespaldoRepository) {}

  async crearRespaldo(destino: string): Promise<RespaldoResult> {
    return await this.repository.crearRespaldo(destino);
  }

  async getInfo(): Promise<RespaldoInfo> {
    return await this.repository.getInfo();
  }

  /** Reemplaza todos los datos por los de una copia anterior. Deja la sesión
   * sin uso: el usuario con el que se entró puede no existir en la copia. */
  async restaurarRespaldo(origen: string): Promise<RestauracionResult> {
    return await this.repository.restaurarRespaldo(origen);
  }
}
