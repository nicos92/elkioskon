import type { NocturnoConfig } from "../../domain/entities";

export interface INocturnoConfigRepository {
  getConfig(): Promise<NocturnoConfig>;
  saveConfig(config: NocturnoConfig): Promise<void>;
}