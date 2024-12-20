# Swing 3D

## Licencia
Este proyecto está protegido por derechos de autor. No está permitido usar, copiar, modificar o distribuir este software sin autorización previa por escrito del autor.


## ¿Qué es?

Swing 3D es un proyecto de la Universidad Politécnica de Madrid, pensado originalmente por la Facultad de Ciencias de la Actividad Física y el Deporte - INEF, y llevado a cabo por la Escuela Técnica Superior de Ingenieros de Telecomunicación, concretamente por el grupo de investigación GAMMA. 

El objetivo de este proyecto es poder representar y analizar los movimientos grabados con cámaras Vicon en sistemas tridimensionales. Esta orientado a un público especializado en deporte — originalmente golf — que no tiene por qué tener conocimientos técnicos.

## Demo

![image](https://github.com/user-attachments/assets/cc15e3f3-833a-4590-b7d0-7eaa59e1e46d)


## Especificaciones técnicas

### Rust

Al proyecto se le impone una especificación técnica desde un inicio: Debe tener una versión web. Por tanto, como lenguaje de programación para el proyecto se descartan la mayoría de lenguajes de programación típicos. Entre Javascript y Rust se elige Rust por su velocidad, escalabilidad y estabilidad. 

### WASM

En el momento de redacción de estas líneas (3/10/2024), WASM no tiene acceso al DOM, sino que se "virtualiza" el código en un entorno JavaScript, y se cuenta con las limitaciones del navegador (la más restictiva es que únicamente se cuenta con un hilo de procesamiento). Esto implica que Vanilla JavaScript consigue una ligera ventaja sobre Web Assembly en prácticamente la totalidad de los benchmarks. Pero la diferencia es mínima, y hay que tener en cuenta que en ningún caso se utilizaría Vanilla JS para este proyecto. Las alternativas estudiadad son Babylon JS y Three JS, que posiblemente rendirán peor que un motor de Rust. Además, hay que tener en cuenta que el proyecto cuenta con una versión de escritorio, donde Rust es infinitamente más rápido. 

### Bevy

Pese a no ser estrictamente necesario, se opta por elegir un motor gráfico para el proyecto, para eliminar la necesidad de lidiar con la gpu. Se elige Bevy principalmente por dos motivos:
* **Está escrito plenamente en Rust:** Se puede compilar de forma nativa a Web Assembly.
* **Es completamente modular:** Permite crear webs relativamente ligeras.

Además, este motor gráfico está muy optimizado, posiblemente consiga tasas de frames muy similares a las que se conseguirían sin utilizar un motor gráfico. 

Bevy utiliza el patrón de diseño _ECS_, que es muy simple, pero diferente a la orientación a objetos, conviene familiarizarse. 

Bevy tiene el contraparte de estar aún en desarrollo, por lo que sus versiones estables cambian constantemente, pero ofrecen guías para actualizar. Además, la comunidad es muy amplia y los plugins y assets son numerosos. 


# Objetivos del proyecto

- [x] Leer ficheros C3D
- [x] Leer ficheros de configuración
- [ ] Editar ficheros C3D
- [ ] Editar ficheros de configuración
- [x] Representar el fichero C3D
- [x] Crear un avatar personalizable por el usuario
- [ ] Representar vectores de fuerzas en el entorno 3D
- [x] Añadir controles de tiempo sobre un C3D (_play/pause_ y control del _frame_ )
- [x] Añadir un eje de tiempo
- [x] Añadir una escena 3D y una cámara
- [ ] Posibilidad de cambiar la escena en _runtime_
- [x] Calcular y representar datos biomecánicos en gráficas 2D
- [ ] Calcular y representar datos biomecánicos en el entorno 3D


