#include <linux/init.h>    // Macros utilizadas para marcar las funciones de inicialización y limpieza
#include <linux/module.h>  // Contiene las definiciones de las funciones y macros para módulos
#include <linux/kernel.h>  // Contiene las definiciones de las funciones y macros para el kernel

// Función que se ejecuta al cargar el módulo
static int __init hello_init(void) {
    printk(KERN_INFO "Hola, mundo!\n");
    return 0;    // Retorno no negativo significa que el módulo se cargó correctamente
}

// Función que se ejecuta al descargar el módulo
static void __exit hello_exit(void) {
    printk(KERN_INFO "Adiós, mundo!\n");
}

// Macros para registrar las funciones de inicialización y limpieza
module_init(hello_init);
module_exit(hello_exit);

// Información del módulo
MODULE_LICENSE("GPL");
MODULE_AUTHOR("Tu Nombre");
MODULE_DESCRIPTION("Un módulo simple que imprime mensajes en el kernel log");
MODULE_VERSION("0.1");
