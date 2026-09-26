import type { RespaldoInfo, RespaldoResult } from "../../domain/entities";
import type { IRespaldoRepository } from "../../domain/interfaces";

export class RespaldoUseCase {
  constructor(private repository: IRespaldoRepository) {}

  async crearRespaldo(destino: string): Promise<RespaldoResult> {
    return await this.repository.crearRespaldo(destino);
  }

  async getInfo(): Promise<RespaldoInfo> {
    return await this.repository.getInfo();
  }
}
