


use std::process::Command;

pub fn get_connected_networks()->Option<Vec<String>>{
    let netsh_show_connected : String = String::from("netsh wlan show interfaces");
    let output = Command::new("cmd")
            .args(&["/C", netsh_show_connected.as_str()])
            .output()
            .expect("failed to execute process");
    let str_output = match String::from_utf8(output.stdout) {
        Ok(output) => output,
        Err(_) => String::from("failed")
    };
    
    let mut ssids:Vec<String> = Vec::new();
    for line in str_output.split_terminator('\n') {
        let trimmed = line.trim();

        if trimmed.starts_with("SSID"){
            
            let parts: Vec<&str> = trimmed.split(":").collect();
            ssids.push(String::from(parts[1].trim()));
        }
    }
    if ssids.len()>0 {
        return Option::Some(ssids)
    }else{
        return Option::None
    }
}