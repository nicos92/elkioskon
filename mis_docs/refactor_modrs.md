# Refactorización del módulo `infrastructure/database`

## Contexto

El proyecto es una aplicación de escritorio desarrollada con:

* **Tauri 2.0**
* **Backend en Rust**
* **Frontend en Vue.js + TypeScript**
* Arquitectura orientada a **Clean Architecture**
* El código relacionado con persistencia se encuentra dentro de `infrastructure/database`.

Actualmente existe un archivo:

```text
src-tauri/
└── src/
    └── infrastructure/
        └── database/
            └── mod.rs
```

El archivo `mod.rs` contiene actualmente múltiples responsabilidades relacionadas con la base de datos.

Quiero realizar una **refactorización estructural**, aplicando principios **SOLID**, con especial énfasis en **Single Responsibility Principle (SRP)**, inversión de dependencias y testabilidad.

---

## Objetivo principal

Analiza exhaustivamente `infrastructure/database/mod.rs` y determina cómo dividirlo en módulos y archivos más pequeños, cohesivos y fáciles de probar.

El objetivo **NO es simplemente separar código en varios archivos**, sino identificar correctamente las responsabilidades existentes y establecer límites claros entre ellas.

La nueva estructura debe:

* Reducir la responsabilidad de `mod.rs`.
* Separar responsabilidades relacionadas con:

  * configuración de la base de datos;
  * creación y configuración de conexiones/pools;
  * ejecución de queries;
  * repositorios;
  * modelos específicos de persistencia;
  * mapeo entre modelos de infraestructura y entidades del dominio, cuando corresponda;
  * errores de infraestructura;
  * migraciones, si actualmente están gestionadas desde este módulo;
  * cualquier otra responsabilidad que actualmente esté mezclada.
* Facilitar la creación de **unit tests**.
* Reducir el acoplamiento.
* Mejorar la inyección de dependencias.
* Aplicar correctamente **SOLID**.
* Mantener una separación clara entre infraestructura y dominio/aplicación.
* Evitar que el dominio dependa directamente de detalles de implementación de la base de datos.

---

# Antes de modificar código

No comiences inmediatamente a editar archivos.

Primero realiza un análisis del código actual de `infrastructure/database/mod.rs`.

Identifica explícitamente:

1. Todas las responsabilidades que actualmente posee `mod.rs`.
2. Qué partes del código están fuertemente acopladas.
3. Qué dependencias concretas utiliza.
4. Qué código es difícil de testear actualmente y por qué.
5. Qué principios SOLID se están incumpliendo.
6. Qué abstracciones deberían existir.
7. Qué dependencias deberían invertirse.
8. Qué componentes deberían permanecer juntos porque pertenecen a la misma responsabilidad.
9. Qué componentes deberían separarse porque tienen diferentes motivos de cambio.
10. Qué parte pertenece realmente a `infrastructure` y qué parte debería pertenecer a `application` o `domain`.

No crees abstracciones innecesarias únicamente para "cumplir SOLID".

La prioridad debe ser:

> **alta cohesión + bajo acoplamiento + responsabilidades claras + testabilidad**

y no aumentar artificialmente la cantidad de interfaces o archivos.

---

# Diseñar primero la nueva estructura

Antes de implementar la refactorización, propón una estructura de archivos concreta.

Por ejemplo, evalúa si una estructura similar a esta tiene sentido:

```text
infrastructure/
└── database/
    ├── mod.rs
    ├── connection.rs
    ├── config.rs
    ├── error.rs
    ├── repositories/
    │   ├── mod.rs
    │   └── ...
    ├── models/
    │   ├── mod.rs
    │   └── ...
    └── mappers/
        ├── mod.rs
        └── ...
```

**No asumas que esta estructura es correcta.**

Adáptala al código real del proyecto.

Si consideras que una estructura diferente es mejor, utilízala y explica por qué.

---

# Responsabilidad de `mod.rs`

Después de la refactorización, `mod.rs` debería actuar principalmente como **punto de composición y exposición del módulo**, no como un archivo que contenga toda la lógica de infraestructura de base de datos.

Evita que `mod.rs` vuelva a convertirse en un "God Module".

La lógica concreta debe vivir en módulos especializados.

---

# SOLID

Analiza específicamente los siguientes principios:

## SRP — Single Responsibility Principle

Cada módulo, struct y componente debe tener una responsabilidad clara y un motivo de cambio razonablemente único.

Busca especialmente:

* structs que hagan demasiadas cosas;
* funciones demasiado grandes;
* código de conexión mezclado con queries;
* queries mezcladas con mapeos;
* configuración mezclada con lógica;
* manejo de errores mezclado con lógica de negocio;
* repositorios que conozcan detalles que no deberían conocer.

---

## OCP — Open/Closed Principle

Determina si existen componentes que puedan diseñarse para permitir nuevas implementaciones sin modificar código existente.

No fuerces este principio si no aporta valor real.

---

## LSP — Liskov Substitution Principle

Si existen traits e implementaciones concretas, verifica que las implementaciones puedan sustituirse correctamente sin modificar el comportamiento esperado.

---

## ISP — Interface Segregation Principle

Evita traits gigantes.

Si existe una abstracción como:

```rust
trait DatabaseRepository {
    // muchas operaciones diferentes
}
```

evalúa si debería dividirse en interfaces más pequeñas y específicas.

Por ejemplo:

```rust
trait UserRepository {
    // operaciones relacionadas con usuarios
}
```

y otras abstracciones específicas cuando corresponda.

No introduzcas interfaces artificiales.

---

## DIP — Dependency Inversion Principle

Este principio es especialmente importante.

Analiza si las capas superiores dependen directamente de implementaciones concretas como:

```rust
SqlitePool
```

u otros detalles específicos del motor de base de datos.

Cuando corresponda, establece abstracciones mediante traits y permite que la implementación concreta de infraestructura dependa de esas abstracciones.

Por ejemplo, conceptualmente:

```text
Application
    ↓
Trait / Port
    ↑
Infrastructure
    ↓
SQLite / SQLx
```

La implementación concreta de la base de datos no debería filtrarse innecesariamente hacia las capas superiores.

---

# Testabilidad

Uno de los objetivos principales de esta refactorización es facilitar los **unit tests**.

Para cada componente identifica:

* qué dependencia externa utiliza;
* si puede probarse sin acceder a una base de datos real;
* si necesita un trait;
* si puede utilizarse un mock/fake;
* si debería ser un unit test o integration test.

Por ejemplo, si un caso de uso depende directamente de:

```rust
SqlitePool
```

evalúa si sería mejor que dependiera de:

```rust
UserRepository
```

mientras que la implementación concreta:

```text
SqliteUserRepository
```

permanezca en infrastructure.

La intención es poder realizar pruebas similares a:

```text
UseCase
   ↓
MockUserRepository
```

sin necesidad de levantar SQLite.

---

# No sobre-abstraer

No crees traits simplemente porque "SOLID dice que hay que usar interfaces".

En Rust, utiliza traits cuando exista una razón concreta, especialmente:

* inversión de dependencias;
* sustitución de implementaciones;
* testabilidad;
* desacoplamiento entre capas.

Si una abstracción no aporta ninguno de estos beneficios, considera mantener la implementación concreta.

---

# Dependencias

Analiza cuidadosamente el grafo de dependencias.

El resultado debería evitar dependencias innecesarias como:

```text
Domain
   ↓
Infrastructure
   ↓
Database
```

cuando conceptualmente debería ser:

```text
Domain
   ↑
Application
   ↑
Infrastructure
   ↓
Database
```

Busca dependencias circulares y dependencias que atraviesen incorrectamente los límites de la arquitectura.

---

# Errores

Analiza el manejo actual de errores.

Determina:

* qué errores son propios de infraestructura;
* cuáles deberían convertirse a errores de aplicación;
* dónde deberían definirse los tipos de error;
* cómo evitar filtrar detalles innecesarios de SQLite/SQLx hacia capas superiores.

Evita utilizar `Box<dyn Error>` como solución genérica si puede diseñarse un error tipado apropiadamente.

Mantén el manejo de errores idiomático de Rust.

---

# Compatibilidad

La refactorización debe mantener el comportamiento actual de la aplicación.

No cambies:

* reglas de negocio;
* comportamiento de los comandos Tauri;
* contratos públicos existentes;
* consultas SQL salvo que sea estrictamente necesario;
* nombres de tablas;
* estructura de datos;
* comportamiento del frontend.

El objetivo inicial es **refactorizar la estructura**, no agregar funcionalidades.

---

# Proceso de implementación

Trabaja en etapas.

### Etapa 1 — Análisis

Analiza el `mod.rs` actual y documenta sus responsabilidades.

### Etapa 2 — Diseño

Propón la nueva estructura de módulos y explica:

* qué responsabilidad tendrá cada archivo;
* qué structs tendrá;
* qué traits tendrá;
* quién dependerá de quién;
* qué dependencias se invertirán.

### Etapa 3 — Refactorización

Implementa la estructura propuesta.

Mueve el código progresivamente y evita realizar una reescritura completa innecesaria.

### Etapa 4 — Corrección de dependencias

Actualiza imports, módulos, `use`, visibilidad (`pub`, `pub(crate)`, privada) y referencias.

Evita hacer públicos elementos que no necesitan serlo.

### Etapa 5 — Validación

Después de la refactorización ejecuta:

```bash
cargo check
cargo test
cargo clippy
```

Si el proyecto tiene comandos adicionales de validación, ejecútalos también.

---

# Tests

Una vez terminada la refactorización, analiza qué componentes deberían tener tests.

No generes tests artificiales únicamente para aumentar la cobertura.

Prioriza:

1. lógica que pueda ejecutarse sin infraestructura externa;
2. mapeos;
3. validaciones;
4. manejo de errores;
5. repositorios mediante abstracciones/mock cuando corresponda;
6. integración con SQLite para los casos que realmente requieran una base de datos.

Diferencia claramente:

```text
Unit Test
```

de:

```text
Integration Test
```

No intentes convertir una prueba de integración con SQLite en un unit test artificial.

---

# Criterios de aceptación

Considera que la refactorización está correctamente realizada cuando:

* `database/mod.rs` deja de contener múltiples responsabilidades.
* Cada módulo tiene una responsabilidad clara.
* Las dependencias están correctamente orientadas.
* Las capas superiores no dependen innecesariamente de detalles concretos de SQLite/SQLx.
* Las abstracciones existentes tienen una razón clara.
* No existen traits creados únicamente por cumplir SOLID.
* Los componentes importantes pueden probarse aisladamente.
* Se redujo el acoplamiento.
* Se aumentó la cohesión.
* No se modificó el comportamiento funcional existente.
* `cargo check` finaliza correctamente.
* `cargo test` finaliza correctamente.
* `cargo clippy` no introduce nuevos problemas relevantes.

---

# Importante

Antes de implementar cualquier cambio, **muéstrame primero el análisis de `mod.rs` y la estructura propuesta**.

No empieces creando archivos sin haber determinado primero las responsabilidades y dependencias.

Quiero que la refactorización esté basada en el código real del proyecto y no en una estructura genérica de Clean Architecture.

La prioridad es:

**comprender → analizar responsabilidades → diseñar dependencias → refactorizar → probar → validar**

No quiero una simple separación física del archivo.

Quiero una **refactorización arquitectónica orientada a SOLID, bajo acoplamiento, alta cohesión y testabilidad**.
