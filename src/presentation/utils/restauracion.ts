const CLAVE = "respaldoPrevioRestauracion";

/**
 * A restore leaves the app logged out, so the path of the safety copy cannot
 * travel in the store or in the query string: the first navigation to the login
 * screen would be enough to lose it, and a path squeezed into a URL is one
 * encoding mistake away from showing the user something that does not exist.
 */
export function guardarRutaRespaldoPrevio(ruta: string | null) {
    if (ruta) {
        sessionStorage.setItem(CLAVE, ruta);
    }
}

export function leerRutaRespaldoPrevio(): string | null {
    const ruta = sessionStorage.getItem(CLAVE);
    sessionStorage.removeItem(CLAVE);
    return ruta;
}
