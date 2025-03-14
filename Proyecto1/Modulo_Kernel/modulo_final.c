#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/slab.h>
#include <linux/uaccess.h>
#include <linux/fs.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("202002873");
MODULE_DESCRIPTION("Módulo para capturar métricas necesarias para el servicio en rust");
MODULE_VERSION("1.0");

#define PROC_NAME "sysinfo_202002873"
#define DATA_FILE "/tmp/docker_stats.json"

static int show_sysinfo(struct seq_file *m, void *v) {
    struct file *file;
    char *buf;
    loff_t pos = 0;
    int ret;

    // Abrir el archivo
    file = filp_open(DATA_FILE, O_RDONLY, 0);
    if (IS_ERR(file)) {
        seq_printf(m, "Error: No se pudo abrir el archivo %s\n", DATA_FILE);
        return PTR_ERR(file);
    }

    buf = kzalloc(PAGE_SIZE, GFP_KERNEL);
    if (!buf) {
        seq_printf(m, "Error: No se pudo asignar memoria\n");
        filp_close(file, NULL);
        return -ENOMEM;
    }

    ret = kernel_read(file, buf, PAGE_SIZE, &pos);
    if (ret < 0) {
        seq_printf(m, "Error: No se pudo leer el archivo\n");
    } else {
        seq_printf(m, "%s\n", buf);
    }

    // Liberar recursos
    kfree(buf);
    filp_close(file, NULL);
    return 0;
}

// Función para abrir el archivo en /proc
static int sysinfo_open(struct inode *inode, struct file *file) {
    return single_open(file, show_sysinfo, NULL);
}

// Operaciones del archivo en /proc
static const struct proc_ops sysinfo_fops = {
    .proc_open = sysinfo_open,
    .proc_read = seq_read,
    .proc_lseek = seq_lseek,
    .proc_release = single_release,
};

static int __init sysinfo_init(void) {
    proc_create(PROC_NAME, 0, NULL, &sysinfo_fops);
    printk(KERN_INFO "Módulo sysinfo_2020 cargado\n");
    return 0;
}

static void __exit sysinfo_exit(void) {
    remove_proc_entry(PROC_NAME, NULL);
    printk(KERN_INFO "Módulo sysinfo_2020 descargado\n");
}

module_init(sysinfo_init);
module_exit(sysinfo_exit);

