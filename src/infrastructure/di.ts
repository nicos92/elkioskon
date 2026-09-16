import type {
  IArticuloRepository,
  IAuditRepository,
  ICategoriaRepository,
  ICierreRepository,
  IClienteRepository,
  IDollarRepository,
  IHomeRepository,
  INocturnoConfigRepository,
  IPresupuestoRepository,
  IProveedorRepository,
  IStockRepository,
  ISubCategoriaRepository,
  ITipoVentaRepository,
  IUserRepository,
  IVentaRepository,
} from "../domain/interfaces";
import { ArticuloApiRepository } from "./api/articuloRepository";
import { AuditApiRepository } from "./api/auditRepository";
import { CategoriaApiRepository } from "./api/categoriaRepository";
import { CierresApiRepository } from "./api/cierreRepository";
import { ClienteApiRepository } from "./api/clienteRepository";
import { DollarApiRepository } from "./api/dollarRepository";
import { HomeApiRepository } from "./api/homeRepository";
import { NocturnoConfigApiRepository } from "./api/nocturnoRepository";
import { PresupuestoApiRepository } from "./api/presupuestoRepository";
import { ProveedorApiRepository } from "./api/proveedorRepository";
import { StockApiRepository } from "./api/stockRepository";
import { SubCategoriaApiRepository } from "./api/subCategoriaRepository";
import { TipoVentaApiRepository } from "./api/tipoVentaRepository";
import { UserApiRepository } from "./api/userRepository";
import { VentasApiRepository } from "./api/ventaRepository";

export const userRepository: IUserRepository = new UserApiRepository();
export const proveedorRepository: IProveedorRepository =
  new ProveedorApiRepository();
export const categoriaRepository: ICategoriaRepository =
  new CategoriaApiRepository();
export const subCategoriaRepository: ISubCategoriaRepository =
  new SubCategoriaApiRepository();
export const articuloRepository: IArticuloRepository =
  new ArticuloApiRepository();
export const stockRepository: IStockRepository = new StockApiRepository();
export const clienteRepository: IClienteRepository =
  new ClienteApiRepository();
export const ventaRepository: IVentaRepository = new VentasApiRepository();
export const tipoVentaRepository: ITipoVentaRepository =
  new TipoVentaApiRepository();
export const presupuestoRepository: IPresupuestoRepository =
  new PresupuestoApiRepository();
export const auditRepository: IAuditRepository = new AuditApiRepository();
export const cierreRepository: ICierreRepository = new CierresApiRepository();
export const homeRepository: IHomeRepository = new HomeApiRepository();
export const dollarRepository: IDollarRepository = new DollarApiRepository();
export const nocturnoConfigRepository: INocturnoConfigRepository =
  new NocturnoConfigApiRepository();
