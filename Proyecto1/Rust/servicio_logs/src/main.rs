use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::process::Command;
use chrono::Utc;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc;
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct SystemInfo {
    #[serde(rename = "ram_total")]
    ram_total: u64,
    #[serde(rename = "ram_libre")]
    ram_libre: u64,
    #[serde(rename = "ram_usada")]
    ram_usada: u64,
    #[serde(rename = "cpu_usado")]
    cpu_usado: f64,
    #[serde(rename = "containers")]
    containers: Vec<Container>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Container {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Nombre")]
    nombre: String,
    #[serde(rename = "Memoria")]
    memoria: f64,
    #[serde(rename = "CPU")]
    cpu: f64,
    #[serde(rename = "Disco")]
    disco: f64,
    #[serde(rename = "IO")]
    io: f64,
    #[serde(rename = "PID")]
    pid: String,
}

#[derive(Debug, Serialize, Clone)]
struct LogContainer {
    pid: String,
    container_id: String,
    nombre: String,
    memoria: f64,
    cpu: f64,
    timestamp: String,
}

fn kill_container(id: &str) -> std::process::Output {
    let output = std::process::Command::new("sudo")
        .arg("docker")
        .arg("rm")
        .arg("-f") 
        .arg(id)
        .output()
        .expect("failed to execute process");

    println!("Eliminando contenedor con id: {}", id);
    output
}

fn analyzer(system_info: SystemInfo) {
    println!("Total RAM: {} MB", system_info.ram_total);
    println!("RAM libre: {} MB", system_info.ram_libre);
    println!("RAM usada: {} MB", system_info.ram_usada);
    println!("CPU usado: {} %", system_info.cpu_usado);
    println!("-------------------------------------");

    let containers_list: Vec<Container> = system_info.containers;
    let (tx, rx) = mpsc::channel(); // Crear un canal para comunicación entre hilos

    // Iterar sobre una referencia a containers_list
    for container in &containers_list {
        let tx = tx.clone(); // Clonar el transmisor para cada hilo
        let container = container.clone(); // Clonar el contenedor para moverlo al hilo
        thread::spawn(move || {
            let log_container = LogContainer {
                pid: container.pid.clone(),
                container_id: container.id.to_string(),
                nombre: container.nombre.clone(),
                memoria: container.memoria,
                cpu: container.cpu,
                timestamp: Utc::now().to_rfc3339(),
            };

            // Eliminar el contenedor
            let _output = kill_container(&container.id);

            // Enviar el log al hilo principal
            tx.send(log_container).unwrap();
        });
    }

    // Recopilar los logs de los hilos
    let mut log_cont_list = Vec::new();
    for _ in 0..containers_list.len() {
        if let Ok(log) = rx.recv() {
            log_cont_list.push(log);
        }
    }

    if !log_cont_list.is_empty() {
        let logs_json = match serde_json::to_string(&log_cont_list) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Error converting logs to JSON: {}", e);
                return;
            }
        };

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "curl -X POST http://localhost:5000/allLogs -H 'Content-Type: application/json' -d '{}'",
                logs_json
            ))
            .output()
            .expect("Failed to execute curl command");

        if output.status.success() {
            println!("Logs enviados exitosamente a la API.");
        } else {
            eprintln!("Error al enviar logs: {:?}", output.stderr);
        }
    } else {
        println!("No hay logs para enviar.");
    }

    println!("-------------------------------------");
}

fn delete_docker_compose() {
    let output = Command::new("sudo")
        .arg("docker")
        .arg("rm")
        .arg("-f")
        .arg("python_container")
        .output()
        .expect("Failed to execute process");

    if output.status.success() {
        println!("Contenedor 'python_container' matado/eliminado");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error al eliminar el contenedor: {}", stderr);
    }
    
}

fn execute_docker_compose() {
    let output = Command::new("docker-compose")
        .arg("-f")
        .arg("/home/francisco/Escritorio/U/LabSopes/Proyectos/Proyecto1/Logs_python/docker-compose.yml")
        .arg("up")
        .arg("--build")  // Reconstruir las imágenes
        .arg("-d")    
        .output()
        .expect("failed to execute docker-compose");

    if output.status.success() {
        println!("Servicios de api con python levantado correctamente con docker-compose.");
    } else {
        eprintln!("Error ejecutando docker-compose: {:?}", output.stderr);
    }
    thread::sleep(Duration::from_secs(4));
}

fn read_proc_file(file_name: &str) -> io::Result<String> {
    let path  = Path::new("/proc").join(file_name);
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn parse_proc_to_struct(json_str: &str) -> Result<SystemInfo, serde_json::Error> {
    let system_info: SystemInfo = serde_json::from_str(json_str)?;
    Ok(system_info)
}

fn send_system_data(system_info: &SystemInfo) {
    let system_json = match serde_json::to_string(&json!({
        "ram_total": system_info.ram_total,
        "ram_libre": system_info.ram_libre,
        "ram_usada": system_info.ram_usada,
        "cpu_usado": system_info.cpu_usado,
    })) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error converting memory data to JSON: {}", e);
            return;
        }
    };
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "curl -X POST http://localhost:5000/system -H 'Content-Type: application/json' -d '{}'",
            system_json
        ))
        .output()
        .expect("Failed to execute curl command");
    
    if output.status.success() {
        println!("Data del sistema enviada correctamente a la API.");
    } else {
        eprintln!("Error al enviar los datos del sistema: {:?}", output.stderr);
    }
    println!("-------------------------------------");
}



fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("Deteniendo el servicio de Rust");
        delete_docker_compose();

        let outputk = Command::new("pkill")
        .arg("-f")
        .arg("/home/francisco/Escritorio/U/LabSopes/Proyectos/Proyecto1/Script/containers.sh")
        .output()
        .expect("Failed to execute pkill command");

        if outputk.status.success() {
            println!("Todos los procesos de containers.sh han sido terminados.");
        } else {
            eprintln!(
                "Error al terminar los procesos de containers.sh: {:?}",
                outputk.stderr
            );
        }


        let output = Command::new("sh")
        .arg("-c")
        .arg("crontab -l | grep -v \"/home/francisco/Escritorio/U/LabSopes/Proyectos/Proyecto1/Script/containers.sh\" | crontab -")
        .output()
        .expect("Failed to execute command");

        if output.status.success() {
            println!("Cronjob eliminado correctamente.");
        } else {
            eprintln!("Error al eliminar la entrada del crontab: {:?}", output.stderr);
        }

        let outputc = Command::new("sh")
        .arg("-c")
        .arg("docker ps -a --format '{{.ID}} {{.Names}}' | awk '$2 != \"grafana\" {print $1}' | xargs docker rm -f")
        .output()
        .expect("Failed to execute command");

        if outputc.status.success() {
            println!("Contenedores eliminados correctamente (excepto 'grafana').");
        } else {
            eprintln!(
                "Error al eliminar contenedores: {:?}",
                String::from_utf8_lossy(&outputc.stderr)
            );
        }




        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    execute_docker_compose();

    while running.load(Ordering::SeqCst) {
        let json_str = match read_proc_file("sysinfo_202002873") {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading proc file: {}", e);
                continue; 
            }
        };

        let system_info = match parse_proc_to_struct(&json_str) {
            Ok(info) => info,
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
                continue; 
            }
        };
        // -----------------------------
        send_system_data(&system_info);
        analyzer(system_info);
        
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}


