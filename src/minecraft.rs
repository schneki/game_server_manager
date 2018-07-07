use std::process::{Command, Child, Stdio};
use std::io::{Read, BufRead, BufReader, BufWriter, Write};

pub struct Mod {
    pub name: String,
    pub data: Vec<u8>
}


pub fn start_server(config: &::Config) -> Result<Child, String>  {
    let mut child = Command::new("java")
        .current_dir(&config.mc_path)
        .args(&["-jar", &config.mc_server_jar, "nogui"])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("fail");

    let reader = BufReader::new(child.stdout.take().unwrap());


    for line in reader.lines() {
        let msg = line.unwrap();
        if msg.contains("[Server thread/INFO] [minecraft/DedicatedServer]: Done") {
            println!("Minecraft Server Started");
            return Ok(child);
        }
        if msg.contains("[Server Shutdown Thread/INFO]") {
            println!("Minecraft Server Could not Start");
            return Err("server could not start".into());
        }
    }
    Err("server error".into())
}

pub fn stop_server(mc: &mut Child) {
     mc.stdin.as_mut().unwrap().write_all(b"stop").unwrap();
     let _ = mc.wait().unwrap();
     println!("Minecraft Server stopped");
}

use std::path::Path;
use std::fs;

pub fn get_mods(mc_path: &str) -> Vec<Mod> {
    let mut mods = Vec::new();
    let mc_path = format!("{}/mods", mc_path);
    let dir = Path::new(&mc_path);
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let mut f = fs::File::open(path.clone()).unwrap();
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer).unwrap();
                mods.push(Mod { 
                    name: path
                    .file_name().unwrap()
                    .to_str().unwrap().to_owned(), 
                    data: buffer}); 
            }
        }
    }
    mods
}

use zip::{ZipWriter, CompressionMethod};
use zip::write::FileOptions;

pub fn zip_mods(mc_path: &str, mods: Vec<Mod>) {
    let f = fs::File::create(format!("{}/mods.zip", mc_path)).unwrap();
    let mut zip = ZipWriter::new(f);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);

    for mc_mod in mods {
        zip.start_file(mc_mod.name, options).unwrap();
        zip.write_all(&*mc_mod.data).unwrap();
    }
    zip.finish().unwrap();
}

pub fn get_mods_zip(mc_path: &str) -> Vec<u8> {
    let mut f = fs::File::open(format!("{}/mods.zip", mc_path)).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    buffer
}

pub fn save_mods_and_get_zip(mc_path: &str) -> Vec<u8> {
    let mods = zip_mods(&mc_path, get_mods(&mc_path));
    get_mods_zip(&mc_path) 
}

pub fn add_mods(m: Vec<Mod>) {

}

pub fn remove_mod(name: &str) {


}

pub fn clear_mods() {

}
