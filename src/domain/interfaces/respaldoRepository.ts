import type {
  RespaldoInfo,
  RespaldoResult,
  RestauracionResult,
} from "../../domain/entities";

export interface IRespaldoRepository {
  crearRespaldo(destino: string): Promise<RespaldoResult>;
  getInfo(): Promise<RespaldoInfo>;
  restaurarRespaldo(origen: string): Promise<RestauracionResult>;
}
