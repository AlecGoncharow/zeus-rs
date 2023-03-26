use crate::graphics::prelude::LabeledEntry;
use core::result::Result::Ok;
use naga::front::wgsl;
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use smallvec::SmallVec;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub type ShaderModules<'a> = SmallVec<[LabeledEntry<'a, wgpu::ShaderModule>; 4]>;

pub struct WgslShaderContext<'a> {
    /// All shaders are expected to use this path as their base dir, any any label associated with
    /// the shaders are the relative path
    pub shader_src_path: std::path::PathBuf,
    pub shader_modules: ShaderModules<'a>,
}

impl<'a> WgslShaderContext<'a> {
    pub fn new(shader_src_path: std::path::PathBuf) -> Self {
        Self {
            shader_src_path,
            shader_modules: ShaderModules::new(),
        }
    }

    pub fn register_module(&mut self, device: &wgpu::Device, path: &'a str) {
        let new_module = Self::make_module(device, &self.shader_src_path, path);
        self.shader_modules.push(LabeledEntry {
            label: path,
            entry: new_module,
        });
    }

    pub fn find_module(&self, label: &str) -> Option<&wgpu::ShaderModule> {
        if let Some(module) = &self
            .shader_modules
            .iter()
            .find(|entry| entry.label == label)
        {
            return Some(&module.entry);
        };

        None
    }

    // @TODO @HACK, this only needs to really refresh specific shaders
    pub fn refresh_modules(&mut self, device: &wgpu::Device) {
        for module in self.shader_modules.iter_mut() {
            let new_module = Self::make_module(device, &self.shader_src_path, module.label);
            module.entry = new_module;
        }
    }

    pub fn make_module(
        device: &wgpu::Device,
        base_dir: &std::path::Path,
        path: &str,
    ) -> wgpu::ShaderModule {
        use std::borrow::Cow;
        let fq_path = base_dir.join(path);
        let src = std::fs::read_to_string(fq_path).unwrap();

        // parse it
        let _ = wgsl::parse_str(&src).unwrap();

        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&path),
            source: wgpu::ShaderSource::Wgsl(Cow::from(src)),
        })
    }
}

pub fn start_hotloader(dirty_flag: Arc<AtomicBool>, shader_path: std::path::PathBuf) {
    tokio::spawn(async move {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher =
            notify::watcher(tx, std::time::Duration::from_millis(500)).expect("hotloader broke");

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

        loop {
            match rx.recv() {
                Ok(event) => match event {
                    DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
                        // @TODO maybe pass it to naga for validation here?
                        println!("[Pantheon][SHADER HOTLOAD] reloading {:?}", path);
                        let src = std::fs::read_to_string(&path).unwrap();
                        if let Err(e) = wgsl::parse_str(&src) {
                            eprintln!("Pantheon][SHADER HOTLOAD] shader parse error: \n{:?}", e);
                            continue;
                        }

                        dirty_flag.store(true, Ordering::Release);
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
