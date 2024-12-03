# Ficheros de configuración del C3D

El propósito de este módulo es gestionar la lectura (y posteriormente la escritura) de los ficheros C3D. 

Se contemplan dos tipos de ficheros de configuración, que deben tener un comportamiento idéntico. Se distinguirán por la extensión, ".toml" o ".mkr"

## Estructura ficheros de configuración ".toml"

Este formato es un estándar, por eso se implementa. Tendrá las siguientes "Keys":

- **[*config_name*]** es la única clave obligatoria, que debe aparecer al menos una vez. Puede ser cualquier valor de **texto**. Esta clave puede aparecer tantas veces como convenga, y el programa dejará representar la configuración que se elija. Tendrá los siguientes campos:
    - **Campos obligatorios:** Necesarios para la representación. 
        - **visible_points**: es un array con los puntos visibles. Si un punto no aparece en este array, no será representado.
        - **joins**: Contiene las uniones que formarán el avatar. Será un _array de arrays_, donde cada sub\_array representará puntos que serán unidos entre sí. Cada sub\_array debe tener 2 o más puntos.  
    
        Cada punto se define por su **etiqueta** del marcador, o su **índice** en el c3d.

    - **Campos opcionales:** Para personalizar el estilo. Se aplica a toda la configuración excepto que se defina una regla de orden mayor.
        - **vectors:** es un array de arrays, donde cada sub\_array representa un vector. Cada sub\_array debe tener 1 punto "ancla" y un vector.
        - **point_color:** color de los puntos
        - **join_color:** color de la unión
        - **line_thickness:** grosor de la unión
        - **point_size:** tamaño del punto

- **[*point_groups*]** permite crear grupos de puntos, que se podrán usar en múltiples configuraciones. Son un array de puntos, definidos por su **etiqueta** del marcador, o su **índice** en el c3d. Tiene el campo *point_group* repetido tantas veces como convenga, que se puede usar tanto en *visible_points* como en joins.

- **[*point_group.config*]** permite cambiar la configuración de un grupo de puntos, con mayor prioridad que la configuración de *config_name*. Es la forma de establecer un color para un punto o un grupo de puntos, sin alterar la configuración de los demás. Tiene los siguientes campos (todos opcionales):
    - **point_color:** color de los puntos del grupo. Solo se tiene en cuenta si *point_group* se utiliza en *visible_points*.
    - **point_size:** tamaño de los puntos del grupo. Solo se tiene en cuenta si *point_group* se utiliza en *visible_points*.
    - **join_color** es el color de la unión de los puntos del grupo. Solo se tiene en cuenta si *point_group* se utiliza en *joins*
    - **line_thickness:** grosor de la unión. Solo se tiene en cuenta si *point_group* se utiliza en *joins*

>**Pro tip:** Cuando el sistema lee la configuración, trata los puntos como expresiones regulares (_regex_), por los que es perfectamente válido insertar una _regex_ en un punto para seleccionar varios. Por defecto se añaden modificadores al punto: `^` y `$`. Si quieres eliminar este comportamiento (que no se añadan estos modificadores), el nombre del punto debe empezar con `_`. Por ejemplo, si tenemos un punto llamado "mkr", podemos seleccionar este punto escribiendo "mkr" en _visible\_points_, o en un _point\_group_. Pero si queremos seleccionar _todos_ los puntos que contengan la cadena "mkr" (incluido un punto llamado "p\_mkr\_1"), escribiremos "\_mkr". Pero asegúrate de que no haya ninguna _regex_ en las _visible\_joins_, de lo contrario, el programa no conocerá el orden para unir los puntos!

## Estructura ficheros de configuración ".mkr"

Este formato sirve para ofrecer compatibilidad con los sitemas Vicon. Aún no está implementado. Debe tener el mismo formato que los ficheros ".mkr" que entiende Vicon.