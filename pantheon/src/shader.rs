use anyhow::*;
use core::result::Result::Ok;
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct ShaderData {
    pub src: String,
    pub src_path: PathBuf,
    pub spv_path: PathBuf,
    pub kind: shaderc::ShaderKind,
}

impl ShaderData {
    pub fn load(src_path: PathBuf) -> Result<Self> {
        let extension = src_path
            .extension()
            .context("File has no extension")?
            .to_str()
            .context("Extension cannot be converted to &str")?;
        let kind = match extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            _ => bail!("Unsupported shader: {}", src_path.display()),
        };

        let src = read_to_string(src_path.clone())?;

        let mut build_path = src_path.parent().unwrap().to_path_buf();
        build_path.push("build/");
        if !build_path.exists() {
            std::fs::create_dir(build_path.clone()).expect("Failed to mkdir");
        }
        build_path.push(src_path.file_name().unwrap());

        let spv_path = build_path.with_extension(format!("{}.spv", extension));

        Ok(Self {
            src,
            src_path,
            spv_path,
            kind,
        })
    }
}

pub struct ShaderContext {
    pub shader_src_path: std::path::PathBuf,
    pub shader_spirv_path: std::path::PathBuf,
}

impl ShaderContext {
    pub fn make_module(&self, device: &wgpu::Device, path: &str) -> wgpu::ShaderModule {
        let fq_path = self.shader_spirv_path.join(path);
        let spirv_source = std::fs::read(fq_path).unwrap();
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&path),
            source: wgpu::util::make_spirv(&spirv_source),
        })
    }
}

pub fn start_hotloader(dirty_flag: Arc<AtomicBool>, shader_path: std::path::PathBuf) {
    tokio::spawn(async move {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher =
            notify::watcher(tx, std::time::Duration::from_millis(500)).expect("water broke");

        let path = std::env::current_dir().unwrap();
        // @TODO possibly rethink how we set the parent path of the shader_path
        let shader_path = if let Some(name) = path.file_name() {
            if name != "zeus-rs" {
                panic!("ruh roh");
            } else {
                let output = path.join(shader_path);
                if !output.is_dir() {
                    panic!("what");
                }
                println!("starting launch sequence");
                output
            }
        } else {
            panic!("what");
        };

        watcher
            .watch(shader_path, RecursiveMode::NonRecursive)
            .unwrap();

        let mut compiler = shaderc::Compiler::new().expect("unable to create shader compiler");
        loop {
            match rx.recv() {
                Ok(event) => match event {
                    DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
                        println!("[Pantheon][SHADER HOTLOAD] recompiling {:?}", path);
                        let shader = ShaderData::load(path).unwrap();
                        let compiled = compiler.compile_into_spirv(
                            &shader.src,
                            shader.kind,
                            &shader.src_path.to_str().unwrap(),
                            "main",
                            None,
                        );

                        match compiled {
                            Ok(compiled) => {
                                std::fs::write(shader.spv_path, compiled.as_binary_u8()).unwrap();
                                dirty_flag.store(true, Ordering::Release);
                            }
                            Err(e) => {
                                eprintln!("[Pantheon][SHADER HOTLOAD][ERROR] {:#?}", e);
                            }
                        }
                    }
                    // nvim triggers this on write
                    DebouncedEvent::NoticeRemove(_) => (),
                    _ => {
                        println!("[Pantheon][SHADER HOTLOAD] unhandled event: {:?}", event);
                    }
                },
                Err(e) => {
                    println!("[Pantheon][SHADER HOTLOAD] watch error: {:?}", e);
                }
            }
        }
    });
}
