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
    //action: String,
    timestamp: String,
}

fn kill_container(id: &str) -> std::process::Output {
    let output = std::process::Command::new("sudo")
        .arg("docker")
        .arg("rm")
        .arg("-f") // Forzar eliminación del contenedor
        .arg(id)
        .output()
        .expect("failed to execute process");

    println!("Eliminando contenedor con id: {}", id);
    output
}

fn analyzer( system_info:  SystemInfo) {

    println!("Total RAM: {} MB", system_info.ram_total);
    println!("RAM libre: {} MB", system_info.ram_libre);
    println!("RAM usada: {} MB", system_info.ram_usada);
    println!("CPU usado: {} %", system_info.cpu_usado);
    println!("-------------------------------------");
    
    let mut log_cont_list: Vec<LogContainer> = Vec::new();
    let mut containers_list: Vec<Container> = system_info.containers;

    for container in containers_list.iter() {
        let log_container = LogContainer {
            pid: container.pid.clone(),
            container_id: container.id.to_string(),
            nombre: container.nombre.clone(),
            memoria: container.memoria,
            cpu: container.cpu,
            //action: "Killed".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        };
        log_cont_list.push(log_container.clone());

        //let _output = kill_container(&container.id);

    }

    let logs_json = match serde_json::to_string(&log_cont_list) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error converting logs to JSON: {}", e);
            return;
        }
    };

    
    let output = std::process::Command::new("sh")
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
  
    
    
    // println!("Contenedores matados");
    // for process in log_proc_list {
    //     println!("PID: {}, Name: {}, Container ID: {}, Memory Usage: {}, CPU Usage: {} \n", process.pid, process.name, process.container_id,  process.memory_usage, process.cpu_usage);
    // }
        
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
        .arg("-d")       // Ejecutar en segundo plano
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
        delete_docker_compose();
        println!("Deteniendo el servicio de Rust");
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
        
        std::thread::sleep(std::time::Duration::from_secs(20));
    }
}


