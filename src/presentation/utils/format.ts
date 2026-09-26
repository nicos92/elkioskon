export function formatMoney(value: number): string {
  return `$${value.toFixed(2)}`;
}

export function formatBytes(bytes: number): string {
  if (bytes <= 0) {
    return "0 KB";
  }
  const unidades = ["B", "KB", "MB", "GB"];
  let valor = bytes;
  let indice = 0;
  while (valor >= 1024 && indice < unidades.length - 1) {
    valor /= 1024;
    indice++;
  }
  const decimales = indice === 0 ? 0 : 1;
  return `${valor.toFixed(decimales)} ${unidades[indice]}`;
}
