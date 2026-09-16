import type { NocturnoConfig } from "../../domain/entities";
import type { INocturnoConfigRepository } from "../../domain/interfaces";

export class NocturnoUseCase {
  constructor(private repository: INocturnoConfigRepository) {}

  async getConfig(): Promise<NocturnoConfig> {
    return await this.repository.getConfig();
  }

  async saveConfig(config: NocturnoConfig): Promise<void> {
    return await this.repository.saveConfig(config);
  }
}