import type {
  RespaldoInfo,
  RespaldoResult,
} from "../../domain/entities";

export interface IRespaldoRepository {
  crearRespaldo(destino: string): Promise<RespaldoResult>;
  getInfo(): Promise<RespaldoInfo>;
}
