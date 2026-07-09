#[cfg(target_os = "windows")]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[allow(unused_imports)]
use std::process::Command;

#[cfg(target_os = "windows")]
fn create_hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd
}
use anyhow::{anyhow, Result};
use log::{info, warn};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineInfo {
    pub mac: String,
    pub motherboard: String,
    pub cpu: String,
    pub disk: String,
    pub version: String,
    pub machine_id: String, // 新增：复合机器唯一标识
}

impl Default for MachineInfo {
    fn default() -> Self {
        Self {
            mac: "————".to_string(),
            motherboard: "————".to_string(),
            cpu: "————".to_string(),
            disk: "————".to_string(),
            version: "2.1.0".to_string(),
            machine_id: "————".to_string(),
        }
    }
}

pub async fn get_machine_info() -> Result<MachineInfo> {
    info!("开始获取机器码信息");

    let mut machine_info = MachineInfo::default();

    // 获取MAC地址
    machine_info.mac = get_mac_address().await.unwrap_or_else(|e| {
        warn!("获取MAC地址失败: {}", e);
        "————".to_string()
    });

    // 根据操作系统获取不同的硬件信息
    #[cfg(target_os = "windows")]
    {
        machine_info.motherboard = get_windows_motherboard_serial().await.unwrap_or_else(|e| {
            warn!("获取主板序列号失败: {}", e);
            "————".to_string()
        });

        machine_info.cpu = get_windows_cpu_serial().await.unwrap_or_else(|e| {
            warn!("获取CPU序列号失败: {}", e);
            "————".to_string()
        });

        machine_info.disk = get_windows_disk_serial().await.unwrap_or_else(|e| {
            warn!("获取硬盘序列号失败: {}", e);
            "————".to_string()
        });
    }

    #[cfg(target_os = "macos")]
    {
        machine_info.motherboard = get_macos_motherboard_serial().await.unwrap_or_else(|e| {
            warn!("获取主板序列号失败: {}", e);
            "————".to_string()
        });

        machine_info.cpu = get_macos_cpu_serial().await.unwrap_or_else(|e| {
            warn!("获取CPU序列号失败: {}", e);
            "————".to_string()
        });

        machine_info.disk = get_macos_disk_serial().await.unwrap_or_else(|e| {
            warn!("获取硬盘序列号失败: {}", e);
            "————".to_string()
        });
    }

    #[cfg(target_os = "linux")]
    {
        // 银河麒麟系统目前仅支持获取MAC地址
        machine_info.motherboard = "————".to_string();
        machine_info.cpu = "————".to_string();
        machine_info.disk = "————".to_string();

        info!("银河麒麟系统，仅支持获取MAC地址");
    }

    // 生成复合机器唯一标识
    machine_info.machine_id = generate_machine_id(&machine_info).await;

    info!("机器码信息获取完成: {:?}", machine_info);
    Ok(machine_info)
}

async fn get_mac_address() -> Result<String> {
    match mac_address::get_mac_address() {
        Ok(Some(mac)) => {
            let mac_str = format!(
                "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac.bytes()[0],
                mac.bytes()[1],
                mac.bytes()[2],
                mac.bytes()[3],
                mac.bytes()[4],
                mac.bytes()[5]
            );
            Ok(mac_str)
        }
        Ok(None) => Err(anyhow!("未找到网卡")),
        Err(e) => Err(anyhow!("获取MAC地址失败: {}", e)),
    }
}

#[cfg(target_os = "windows")]
fn clean_hardware_value(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"').to_string();
    if value.is_empty() {
        return None;
    }

    let lower = value.to_ascii_lowercase();
    let invalid_values = [
        "to be filled by o.e.m.",
        "to be filled by oem",
        "default string",
        "system serial number",
        "none",
        "unknown",
        "not applicable",
        "n/a",
    ];

    if invalid_values.iter().any(|invalid| lower == *invalid) {
        return None;
    }

    Some(value)
}

#[cfg(target_os = "windows")]
fn query_wmi<T>(query: &str) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let com = wmi::COMLibrary::new().map_err(|e| anyhow!("初始化WMI失败: {}", e))?;
    let connection = wmi::WMIConnection::new(com).map_err(|e| anyhow!("连接WMI失败: {}", e))?;
    connection
        .raw_query(query)
        .map_err(|e| anyhow!("执行WMI查询失败: {}", e))
}

#[cfg(target_os = "windows")]
fn query_wmi_first_valid<T, F>(query: &str, mut selector: F) -> Result<String>
where
    T: DeserializeOwned,
    F: FnMut(&T) -> Option<String>,
{
    let rows: Vec<T> = query_wmi(query)?;
    rows.iter()
        .find_map(|row| selector(row).and_then(|value| clean_hardware_value(&value)))
        .ok_or_else(|| anyhow!("WMI查询未返回有效值"))
}

#[cfg(target_os = "windows")]
fn get_wmic_value(args: &[&str], key: &str) -> Result<String> {
    let output = create_hidden_command("wmic").args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("wmic执行失败: {}", stderr.trim()));
    }

    let prefix = format!("{}=", key);
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.starts_with(&prefix) {
            let value = line.replace(&prefix, "");
            if let Some(value) = clean_hardware_value(&value) {
                return Ok(value);
            }
        }
    }

    Err(anyhow!("wmic未返回有效值: {}", key))
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WmiBaseBoard {
    serial_number: Option<String>,
    manufacturer: Option<String>,
    product: Option<String>,
    version: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WmiProcessor {
    processor_id: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WmiDiskDrive {
    serial_number: Option<String>,
    index: Option<u32>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WmiComputerSystemProduct {
    #[serde(rename = "UUID")]
    uuid: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WmiBios {
    serial_number: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WmiComputerSystem {
    name: Option<String>,
}

#[cfg(target_os = "windows")]
async fn get_windows_motherboard_serial() -> Result<String> {
    if let Ok(serial) = query_wmi_first_valid::<WmiBaseBoard, _>(
        "SELECT SerialNumber FROM Win32_BaseBoard",
        |board| board.serial_number.clone(),
    ) {
        return Ok(serial);
    }

    if let Ok(serial) = get_wmic_value(
        &["baseboard", "get", "serialnumber", "/value"],
        "SerialNumber",
    ) {
        return Ok(serial);
    }

    // 如果主板序列号无效，尝试获取主板产品信息作为替代
    if let Ok(product_info) = get_motherboard_product_info().await {
        return Ok(format!("MB-{}", product_info));
    }

    Err(anyhow!("未获取到有效的主板标识"))
}

#[cfg(target_os = "windows")]
async fn get_windows_cpu_serial() -> Result<String> {
    if let Ok(serial) = query_wmi_first_valid::<WmiProcessor, _>(
        "SELECT ProcessorId FROM Win32_Processor",
        |processor| processor.processor_id.clone(),
    ) {
        return Ok(serial);
    }

    get_wmic_value(&["cpu", "get", "processorid", "/value"], "ProcessorId")
        .map_err(|_| anyhow!("未获取到CPU序列号"))
}

#[cfg(target_os = "windows")]
async fn get_windows_disk_serial() -> Result<String> {
    if let Ok(mut drives) =
        query_wmi::<WmiDiskDrive>("SELECT Index, SerialNumber FROM Win32_DiskDrive")
    {
        drives.sort_by_key(|drive| drive.index.unwrap_or(u32::MAX));
        for drive in drives {
            if let Some(serial) = drive
                .serial_number
                .as_deref()
                .and_then(clean_hardware_value)
            {
                return Ok(serial);
            }
        }
    }

    get_wmic_value(
        &["diskdrive", "get", "serialnumber", "/value"],
        "SerialNumber",
    )
    .map_err(|_| anyhow!("未获取到硬盘序列号"))
}

#[cfg(target_os = "macos")]
async fn get_macos_motherboard_serial() -> Result<String> {
    // 方法1：尝试获取硬件UUID
    let output = Command::new("system_profiler")
        .args(&["SPHardwareDataType", "-detailLevel", "basic"])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    // 查找Hardware UUID（更可靠）
    for line in output_str.lines() {
        if line.trim().starts_with("Hardware UUID:") {
            let uuid = line
                .split(':')
                .nth(1)
                .ok_or_else(|| anyhow!("解析UUID失败"))?
                .trim()
                .to_string();
            if !uuid.is_empty() {
                return Ok(uuid);
            }
        }
    }

    // 方法2：如果没有UUID，尝试获取序列号
    for line in output_str.lines() {
        if line.trim().starts_with("Serial Number:") {
            let serial = line
                .split(':')
                .nth(1)
                .ok_or_else(|| anyhow!("解析序列号失败"))?
                .trim()
                .to_string();
            if !serial.is_empty() && serial != "(system)" {
                return Ok(serial);
            }
        }
    }

    // 方法3：使用ioreg命令获取主板序列号
    let ioreg_output = Command::new("ioreg")
        .args(&["-c", "IOPlatformExpertDevice", "-d", "2"])
        .output()?;

    let ioreg_str = String::from_utf8_lossy(&ioreg_output.stdout);
    for line in ioreg_str.lines() {
        if line.contains("IOPlatformSerialNumber") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start != end {
                        let serial = &line[start + 1..end];
                        if !serial.is_empty() {
                            return Ok(serial.to_string());
                        }
                    }
                }
            }
        }
    }

    Err(anyhow!("未获取到主板序列号"))
}

#[cfg(target_os = "macos")]
async fn get_macos_cpu_serial() -> Result<String> {
    let output = Command::new("sysctl")
        .args(&["-n", "machdep.cpu.brand_string"])
        .output()?;

    let cpu_info = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !cpu_info.is_empty() {
        // 对CPU信息进行哈希处理作为唯一标识
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(cpu_info.as_bytes());
        let result = hasher.finalize();
        Ok(hex::encode(&result[..8])) // 取前8字节作为标识
    } else {
        Err(anyhow!("未获取到CPU信息"))
    }
}

#[cfg(target_os = "macos")]
async fn get_macos_disk_serial() -> Result<String> {
    let output = Command::new("diskutil").args(&["info", "/"]).output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.trim().starts_with("Volume UUID:") {
            let uuid = line
                .split(':')
                .nth(1)
                .ok_or_else(|| anyhow!("解析UUID失败"))?
                .trim()
                .to_string();
            if !uuid.is_empty() {
                return Ok(uuid);
            }
        }
    }

    Err(anyhow!("未获取到硬盘序列号"))
}

/// 生成复合机器唯一标识
/// 结合多个硬件信息，即使单个信息重复也能保证整体唯一性
async fn generate_machine_id(machine_info: &MachineInfo) -> String {
    let mut components = Vec::new();

    // 1. MAC地址（网卡硬件地址，通常唯一）
    if !machine_info.mac.is_empty() && machine_info.mac != "————" {
        components.push(format!("MAC:{}", machine_info.mac));
    }

    // 2. CPU序列号/ID
    if !machine_info.cpu.is_empty() && machine_info.cpu != "————" {
        components.push(format!("CPU:{}", machine_info.cpu));
    }

    // 3. 硬盘序列号
    if !machine_info.disk.is_empty() && machine_info.disk != "————" {
        components.push(format!("DISK:{}", machine_info.disk));
    }

    // 4. 主板序列号（即使是Default string也包含，作为标识的一部分）
    if !machine_info.motherboard.is_empty() && machine_info.motherboard != "————" {
        components.push(format!("MB:{}", machine_info.motherboard));
    }

    // 5. 获取额外的系统信息增强唯一性
    #[cfg(target_os = "windows")]
    {
        // Windows系统UUID
        if let Ok(system_uuid) = get_windows_system_uuid().await {
            components.push(format!("SYS:{}", system_uuid));
        }

        // BIOS序列号
        if let Ok(bios_serial) = get_windows_bios_serial().await {
            components.push(format!("BIOS:{}", bios_serial));
        }

        // 计算机名
        if let Ok(computer_name) = get_windows_computer_name().await {
            components.push(format!("NAME:{}", computer_name));
        }
    }

    // 如果没有获取到任何有效信息，使用时间戳+随机数
    if components.is_empty() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        components.push(format!("FALLBACK:{}", timestamp));
    }

    // 将所有组件连接并生成SHA256哈希
    let combined = components.join("|");
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();

    // 返回16字符的十六进制字符串（足够唯一且不会太长）
    hex::encode(&result[..8]).to_uppercase()
}

#[cfg(target_os = "windows")]
async fn get_windows_system_uuid() -> Result<String> {
    if let Ok(uuid) = query_wmi_first_valid::<WmiComputerSystemProduct, _>(
        "SELECT UUID FROM Win32_ComputerSystemProduct",
        |product| product.uuid.clone(),
    ) {
        if uuid != "FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF" {
            return Ok(uuid);
        }
    }

    let uuid = get_wmic_value(&["csproduct", "get", "uuid", "/value"], "UUID")?;
    if uuid != "FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF" {
        return Ok(uuid);
    }

    Err(anyhow!("未获取到系统UUID"))
}

#[cfg(target_os = "windows")]
async fn get_windows_bios_serial() -> Result<String> {
    if let Ok(serial) =
        query_wmi_first_valid::<WmiBios, _>("SELECT SerialNumber FROM Win32_BIOS", |bios| {
            bios.serial_number.clone()
        })
    {
        return Ok(serial);
    }

    get_wmic_value(&["bios", "get", "serialnumber", "/value"], "SerialNumber")
        .map_err(|_| anyhow!("未获取到BIOS序列号"))
}

#[cfg(target_os = "windows")]
async fn get_windows_computer_name() -> Result<String> {
    if let Ok(name) = query_wmi_first_valid::<WmiComputerSystem, _>(
        "SELECT Name FROM Win32_ComputerSystem",
        |system| system.name.clone(),
    ) {
        return Ok(name);
    }

    get_wmic_value(&["computersystem", "get", "name", "/value"], "Name")
        .map_err(|_| anyhow!("未获取到计算机名"))
}

#[cfg(target_os = "windows")]
async fn get_motherboard_product_info() -> Result<String> {
    // 尝试获取主板制造商和产品型号
    let mut info_parts = Vec::new();

    // 获取主板制造商
    if let Ok(manufacturer) = get_baseboard_manufacturer().await {
        info_parts.push(manufacturer);
    }

    // 获取主板产品型号
    if let Ok(product) = get_baseboard_product().await {
        info_parts.push(product);
    }

    // 获取主板版本
    if let Ok(version) = get_baseboard_version().await {
        info_parts.push(version);
    }

    if info_parts.is_empty() {
        return Err(anyhow!("未获取到主板产品信息"));
    }

    // 将信息组合并生成哈希作为唯一标识
    let combined = info_parts.join("-");
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();

    Ok(format!(
        "{}#{}",
        combined,
        hex::encode(&result[..4]).to_uppercase()
    ))
}

#[cfg(target_os = "windows")]
async fn get_baseboard_manufacturer() -> Result<String> {
    if let Ok(manufacturer) = query_wmi_first_valid::<WmiBaseBoard, _>(
        "SELECT Manufacturer FROM Win32_BaseBoard",
        |board| board.manufacturer.clone(),
    ) {
        return Ok(manufacturer);
    }

    get_wmic_value(
        &["baseboard", "get", "manufacturer", "/value"],
        "Manufacturer",
    )
    .map_err(|_| anyhow!("未获取到主板制造商"))
}

#[cfg(target_os = "windows")]
async fn get_baseboard_product() -> Result<String> {
    if let Ok(product) =
        query_wmi_first_valid::<WmiBaseBoard, _>("SELECT Product FROM Win32_BaseBoard", |board| {
            board.product.clone()
        })
    {
        return Ok(product);
    }

    get_wmic_value(&["baseboard", "get", "product", "/value"], "Product")
        .map_err(|_| anyhow!("未获取到主板产品型号"))
}

#[cfg(target_os = "windows")]
async fn get_baseboard_version() -> Result<String> {
    if let Ok(version) =
        query_wmi_first_valid::<WmiBaseBoard, _>("SELECT Version FROM Win32_BaseBoard", |board| {
            board.version.clone()
        })
    {
        return Ok(version);
    }

    get_wmic_value(&["baseboard", "get", "version", "/value"], "Version")
        .map_err(|_| anyhow!("未获取到主板版本"))
}
