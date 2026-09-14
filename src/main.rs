
use std::env;
use std::{thread, time::Duration};
use std::process::Command;


mod netsh;
fn main() {
    if !cfg!(target_os = "windows"){
        eprintln!("Not running on windows... Ending program.");
        std::process::exit(1);
    }
    let ssid;
    match env::var("FS_name"){
            Ok(setting)=> {ssid = setting;},
            Err(_) => {
                eprintln!("Environment variable FS_name is not set.\nEnding program.");
                std::process::exit(1);
            },
        };

    
    let mut attempts:i32 = 0;
    eprintln!("SSID: {ssid}");
    loop{
        match netsh::get_connected_networks(){
            Some(networks) => {
                if networks.contains(&ssid){
                    println!("Successfully connected to {}",ssid);
                    
                    break;
                }
            },
            None => if attempts>1 {
                println!("");
            },
        }
        attempts += 1;
        println!("Trying to connect to radar: {}",ssid);
        let netsh_connect: String = format!("netsh wlan connect \"{}\"",ssid);
        let output = Command::new("cmd")
                .args(&["/C", netsh_connect.as_str()])
                .output()
                .expect("failed to execute process");
        let str_output = match String::from_utf8(output.stdout) {
            Ok(output) => output,
            Err(_) => String::from("failed")
        };
        print!("{}",str_output);
        println!("Connection is not yet complete. Do not close the program! The program closes once the connection is established.");
        
        
        thread::sleep(Duration::from_millis(2_000));
    }
    
    
    

}
