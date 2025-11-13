# Sistema Solar Procedural en Rust + Software Rasterizer

Este proyecto es una simulación completa de un **sistema solar 3D**, renderizado usando un **rasterizador propio en software** escrito en Rust. Incluye:

- 🌞 **Sol procedural** con shaders basados en ruido (FBM, turbulencias, emisión variable, gradiente por temperatura/intensidad).
- 🌍 **Tierra**, 🌕 **Luna**, 🔴 **Marte**, 🟠 **Júpiter**, 🪐 **Saturno**.
- 🔄 **Órbitas dinámicas** generadas matemáticamente.
- ⭐ **Skybox 3D procedural** con nebulosas volumétricas, estrellas puntuales y estrellas fugaces.
- 🛰️ **Nave controlable en 3D** con orientación y movimiento realista basado en una base ortonormal.
- 🔧 **Vertex shaders y fragment shaders personalizables para cada tipo de planeta**.

Incluyo pronto un video demostrativo 👇

> *(demo.gif)*

---

# 🚀 Controles

### 📦 Movimiento de la nave (en espacio 3D)
Los controles están inspirados en un sistema tipo "six degrees of freedom":

| Acción | Teclas |
|--------|--------|
| Avanzar / Retroceder | **W / S** |
| Strafe derecha / izquierda | **D / A** |
| Subir / Bajar | **SPACE / LSHIFT** |
| Rotación yaw izquierda / derecha | **← / →** |
| Rotación pitch arriba / abajo | **↑ / ↓** |
| Rotación roll | **Q / E** |
| Acercar / Alejar cámara con respecto a la nave | **R / F** |

### ☀️ Controles del shader del Sol
| Acción | Teclas |
|--------|--------|
| Controlar temperatura del sol | **T / G** |
| Controlar intensidad / emisión del sol | **Y / H** |

---

# 🪐 Objetos del Sistema Solar
El proyecto incluye un conjunto de cuerpos celestes renderizados proceduralmente:

- **Sol** — esfera con shader procedural basado en ruido 3D, animación cíclica, flare en el vertex shader y gradiente por temperatura.
- **Tierra** — planeta rocoso con shader *rocky*, iluminado por el Sol.
- **Luna** — usa el mismo shader rocoso con una paleta distinta.
- **Marte** — shader rocoso modificado con tonos rojizos.
- **Júpiter** — shader *stripes* con bandas paralelas al ecuador, turbulencia animada y zonas nubosas.
- **Saturno** — shader stripes + sus anillos generados como un plano texturizado procedural.

Cada planeta posee:
- Transformación independiente.
- Rotación sobre su propio eje.
- Traslación orbital alrededor del Sol.
- Sombra/luz basada en la posición relativa al Sol.

---

# 🌌 Skybox 3D Procedural
El skybox NO es una textura fija: es generado completamente por ruido 3D.

Incluye:

- **Nebulosas volumétricas** basadas en FBM tridimensional.
- **Estrellas puntuales** (no interpoladas), distribuidas en una esfera gigantesca.
- **Estrellas fugaces** animadas según el tiempo.

El color del skybox depende de la dirección del rayo de cámara y se mantiene estable al rotar.

---

# 🎨 Sistema de Shaders
Los planetas usan un sistema de shaders modular:

### 🌞 Shader del Sol
- FBM 3D para turbulencias.
- Emisión dependiente de intensidad.
- Gradiente por temperatura.
- Desplazamiento en vertex shader para efecto flare.
- Animación cíclica controlada por `time`.

### 🪨 Shader Rocky (planetas rocosos)
- Ruido *value* y FBM para texturas.
- Cráteres generados por ruido umbralizado.
- Cráteres **estáticos**, no animados.
- Iluminación estilo Lambert desde el origen.

### 🌀 Shader Stripes (Júpiter / Saturno)
- Bandas paralelas al ecuador usando `obj_pos.y`.
- Distorsión animada por ruido.
- Turbulencias y nubes.
- Manchas/tormentas generadas proceduralmente.

---

# 🧩 Estructura del Proyecto
```
src/
├─ main.rs            # loop principal, inicialización y control
├─ framebuffer.rs     # rasterizador en software, z-buffer, dibujo de pixeles
├─ matrices.rs        # matrices de transformación, proyección, viewport
├─ entity.rs          # estructura de entidades del sistema solar y nave
├─ shaders/
│   ├─ vertex/
│   │   ├─ basic.rs
│   │   └─ solar_flare.rs
│   └─ fragment/
│       ├─ solar.rs
│       ├─ rocky.rs
│       └─ stripes.rs
├─ skybox.rs          # skybox esférico + estrellas 3D + nebulosas FBM
├─ noise.rs           # ruido 1D/2D/3D, FBM, hash
└─ util.rs            # helpers matemáticos y estructuras comunes
```

---

# 🛠️ Compilar y ejecutar
Requiere Rust:
```sh
cargo run --release
```

La versión *release* es MUY recomendada: el rasterizado en CPU es intensivo.

---

# 📹 Video Demo
Cuando tenga la grabación lista, la embeddearé aquí ✨

> ✨ *Pronto: video del sistema solar en acción*

---

# 🤝 Contribuciones
Abiertas para mejoras:
- Optimización del rasterizador
- Nuevos shaders planetarios (hielo, volcanismo, océanos)
- Mejora del flare solar
- Profundidad real para nebulosas
- Mejor sistema de cámara

---

# 📜 Licencia
MIT
