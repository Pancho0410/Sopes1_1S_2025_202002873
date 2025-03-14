#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/mm.h>
#include <linux/sched.h>
#include <linux/timer.h>
#include <linux/jiffies.h>
#include <linux/fs.h>
#include <linux/uaccess.h>
#include <linux/slab.h>
#include <linux/string.h>
#include <linux/fs_struct.h>
#include <linux/fdtable.h>
#include <linux/file.h>
#include <linux/stat.h>
#include <linux/smp.h> // Para num_online_cpus()

MODULE_LICENSE("GPL");
MODULE_AUTHOR("202002873");
MODULE_DESCRIPTION("Módulo para capturar métricas necesarias para el servicio en rust");
MODULE_VERSION("1.0");

#define PROC_NAME "sysinfo_202002873"
#define MAX_CMDLINE_LENGTH 4096
#define CONTAINER_ID_LENGTH 12

/*Función para obtener el ID del contenedor desde la línea de comandos.*/

static char *get_container_id(const char *cmdline) {
    char *id_start = strstr(cmdline, "-id ");
    if (id_start) {
        id_start += 4; // Saltar "-id "
        char *container_id = kmalloc(CONTAINER_ID_LENGTH + 1, GFP_KERNEL);
        if (container_id) {
            strncpy(container_id, id_start, CONTAINER_ID_LENGTH);
            container_id[CONTAINER_ID_LENGTH] = '\0';
            return container_id;
        }
    }
    return NULL;
}

/*
    Función para obtener la línea de comandos de un proceso.
*/
static char *get_process_cmdline(struct task_struct *task) {
    struct mm_struct *mm;
    char *cmdline, *p;
    unsigned long arg_start, arg_end;
    int len;

    cmdline = kmalloc(MAX_CMDLINE_LENGTH, GFP_KERNEL);
    if (!cmdline)
        return NULL;

    mm = get_task_mm(task);
    if (!mm) {
        kfree(cmdline);
        return NULL;
    }

    down_read(&mm->mmap_lock);
    arg_start = mm->arg_start;
    arg_end = mm->arg_end;
    up_read(&mm->mmap_lock);

    len = arg_end - arg_start;
    if (len > MAX_CMDLINE_LENGTH - 1)
        len = MAX_CMDLINE_LENGTH - 1;

    if (access_process_vm(task, arg_start, cmdline, len, 0) != len) {
        mmput(mm);
        kfree(cmdline);
        return NULL;
    }

    cmdline[len] = '\0';

    // Reemplazar caracteres nulos con espacios
    p = cmdline;
    for (int i = 0; i < len; i++) {
        if (p[i] == '\0')
            p[i] = ' ';
    }

    mmput(mm);
    return cmdline;
}

/*
    Función para calcular el uso de disco de un proceso.
*/
static unsigned long get_disk_usage(struct task_struct *task) {
    unsigned long disk_usage = 0;
    struct files_struct *files;
    struct fdtable *fdt;
    struct file *file;
    struct inode *inode;
    struct kstat stat;
    int i;

    files = task->files;
    if (!files)
        return 0;

    rcu_read_lock();
    fdt = files_fdtable(files);
    for (i = 0; i < fdt->max_fds; i++) {
        file = fdt->fd[i];
        if (file) {
            inode = file_inode(file);
            if (inode && S_ISREG(inode->i_mode)) {
                vfs_getattr(&file->f_path, &stat, STATX_SIZE, AT_STATX_SYNC_AS_STAT);
                disk_usage += stat.size;
            }
        }
    }
    rcu_read_unlock();

    return disk_usage;
}

/*
    Función para calcular el uso total de la CPU del sistema.
*/
static unsigned long get_total_cpu_usage(void) {
    struct task_struct *task;
    unsigned long total_time = 0;

    // Sumar el tiempo de CPU de todos los procesos
    for_each_process(task) {
        total_time += task->utime + task->stime;
    }

    // Obtener el número de núcleos de la CPU
    unsigned int num_cpus = num_online_cpus();

    // Calcular el tiempo total disponible (jiffies * número de núcleos)
    unsigned long total_jiffies = jiffies;
    if (total_jiffies == 0) {
        return 0; // Evitar división por cero
    }

    // Calcular el porcentaje de uso de la CPU
    unsigned long cpu_usage = (total_time * 10000) / (total_jiffies * num_cpus);
    //unsigned long cpu_usage = (total_time * 10000) / (total_jiffies) / (num_cpus * 100);

    return cpu_usage; // Multiplicar por 10000 para obtener dos decimales
}

/*
    Función para mostrar la información en formato JSON.
*/
static int sysinfo_show(struct seq_file *m, void *v) {
    struct sysinfo si; // Estructura para obtener información de la memoria
    struct task_struct *task;
    unsigned long total_jiffies = jiffies;
    int first_process = 1;

    // Obtener información de la memoria
    si_meminfo(&si);

    // Calcular el uso total de la CPU
    unsigned long total_cpu_usage = get_total_cpu_usage();

    // Mostrar la información en formato JSON
    seq_printf(m, "{\n");
    seq_printf(m, "  \"total_ram\": %lu,\n", si.totalram * (si.mem_unit / 1024)); // Total de RAM en KB
    seq_printf(m, "  \"free_ram\": %lu,\n", si.freeram * (si.mem_unit / 1024));  // RAM libre en KB
    seq_printf(m, "  \"used_ram\": %lu,\n", (si.totalram - si.freeram) * (si.mem_unit / 1024)); // RAM en uso en KB
    seq_printf(m, "  \"total_cpu_usage\": %lu.%02lu,\n", total_cpu_usage / 100, total_cpu_usage % 100); // Uso de CPU en porcentaje

    // Mostrar información de los contenedores
    seq_printf(m, "  \"containers\": [\n");

    // Iterar sobre todos los procesos
    for_each_process(task) {
        char *cmdline = get_process_cmdline(task);
        if (cmdline) {
            // Filtrar procesos que tengan un ID de contenedor
            char *container_id = get_container_id(cmdline);
            if (container_id) {
                if (!first_process) {
                    seq_printf(m, ",\n");
                } else {
                    first_process = 0;
                }

                // Obtener métricas del proceso
                unsigned long vsz = task->mm ? task->mm->total_vm << (PAGE_SHIFT - 10) : 0;
                unsigned long rss = task->mm ? get_mm_rss(task->mm) << (PAGE_SHIFT - 10) : 0;
                unsigned long totalram = si.totalram * (si.mem_unit / 1024);
                unsigned long mem_usage = (rss * 10000) / totalram;
                unsigned long total_time = task->utime + task->stime;
                unsigned long cpu_usage = (total_time * 10000) / total_jiffies;
                unsigned long disk_usage = get_disk_usage(task);

                // Mostrar información del contenedor
                seq_printf(m, "    {\n");
                seq_printf(m, "      \"id_container\": \"%s\",\n", container_id);
                seq_printf(m, "      \"PID\": %d,\n", task->pid);
                seq_printf(m, "      \"Name\": \"%s\",\n", task->comm);
                seq_printf(m, "      \"MemoryUsage\": %lu.%02lu,\n", mem_usage / 100, mem_usage % 100);
                seq_printf(m, "      \"CPUUsage\": %lu.%02lu,\n", cpu_usage / 100, cpu_usage % 100);
                seq_printf(m, "      \"DiskUsage\": %lu\n", disk_usage);
                seq_printf(m, "    }\n");

                kfree(container_id);
            }
            kfree(cmdline);
        }
    }

    // Fin
    seq_printf(m, "  ]\n");
    seq_printf(m, "}\n");

    return 0;
}

static int sysinfo_open(struct inode *inode, struct file *file) {
    return single_open(file, sysinfo_show, NULL);
}

static const struct proc_ops sysinfo_ops = {
    .proc_open = sysinfo_open,
    .proc_read = seq_read,
};

static int __init sysinfo_init(void) {
    // Crear el archivo en /proc
    proc_create(PROC_NAME, 0, NULL, &sysinfo_ops);
    printk(KERN_INFO "Módulo sysinfo cargado\n");
    return 0;
}

static void __exit sysinfo_exit(void) {
    // Eliminar el archivo en /proc
    remove_proc_entry(PROC_NAME, NULL);
    printk(KERN_INFO "Módulo sysinfo descargado\n");
}

module_init(sysinfo_init);
module_exit(sysinfo_exit);
