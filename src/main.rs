use chrono::Local;
use eframe::egui::{self, Id, Modal};
use egui_extras::{Column, TableBuilder};
use native_dialog::DialogBuilder;
use ordered_hash_map::OrderedHashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use regex::Regex;
use serde_json::{Value, json};
use sevenz_rust2::{Archive, ArchiveEntry, Password, decompress_with_extract_fn};
use zip::ZipArchive;
use std::ffi::OsStr;
use std::fs::remove_dir_all;
use std::io::{self, Read, Seek};
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use std::path::Path;
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    process::Command,
};
use sysinfo::System;

struct SuperPatchApp {
    selected_tab: Tab,
    orderdata: Value,
    vfsdata: VFSTree,
    vfscache: HashMap<String, VFSNode>,
    organizesort: OrganizeSort,
    organizesort_list: Vec<OrganizeSortListEntry>,
    status: Arc<Mutex<String>>,
    settings: Value,
    vfssort: VFSSort,
    vfssort_list: Vec<VFSSortListEntry>,
    vfssort_list_refresh_requested: bool,
    patchdata: Value,
    livedata: LiveData,
}
#[derive(PartialEq)]
enum Tab {
    Organize,
    Files,
    Patch,
}
#[derive(PartialEq, Clone)]
enum OrganizeSort {
    NameAsc,
    NameDesc,
    CategoryAsc,
    CategoryDesc,
    PriorityAsc,
    PriorityDesc,
}
#[derive(PartialEq, Clone)]
enum SettingsEditType {
    None,
    GamePath,
    GameCommand,
}
#[derive(PartialEq, Clone)]
enum OrganizeSortEditType {
    None,
    Name,
    Category,
    Version,
}
#[derive(Clone)]
struct OrganizeSortListEntry {
    enabled: bool,
    name: String,
    category: String,
    version: String,
    priority: i64,
    path: String,
}
#[derive(Clone)]
struct VFSSortListEntry {
    file_type: String,
    extended: bool,
    path: String,
    name: String,
    down: i64,
    conflicts: i64,
    conflicts_active: i64,
    dltx_patches: i64,
    dltx_patches_active: i64,
}
#[derive(Clone)]
struct VFSSort {
    sort_type: VFSSortType,
    expanded: Vec<String>,
    query: String,
}
#[derive(PartialEq, Clone)]
enum VFSSortType {
    All,
    Conflicts,
    DLTXPatches,
}
type VFSTree = BTreeMap<String, VFSNode>;
#[derive(Clone)]
enum VFSNode {
    Dir(VFSTree),
    File(VFSFile),
}

#[derive(Clone)]
struct VFSFile {
    paths: OrderedHashMap<String, String>,
    dltx_patches: HashMap<String, String>,
}
#[derive(Clone)]
struct LiveData {
    wine_modal_open: bool,
    modded_exes_modal_open: bool,
    import_modal_open: bool,
    install_modal_open: bool,
    install_modal_path: PathBuf,
    install_modal_type: InstallModalType,
    install_modal_vfs_data: Option<VFSNode>,
    install_modal_input_data: InstallModalInputData,
    install_modal_install_data: Option<InstallModalInstallData>,
    install_modal_requested_install_index: usize,
    install_modal_requested_install_index_target: SpecificIndexTarget,
    text_input_modal_open: bool,
    text_input_modal_type: TextInputType,
    text_input_modal_value: String,
    settingsedit_type: SettingsEditType,
    settingsedit_value: String,
    organizeedit_type: OrganizeSortEditType,
    organizeedit_index: usize,
    organizeedit_value: String,
}
#[derive(Clone, PartialEq)]
enum TextInputType {
    None,
    RenameFolder,
}
#[derive(Clone, PartialEq)]
enum InstallModalType {
    None,
    Basic,
    Skip,
    Select,
    Wizard
}
#[derive(Clone, PartialEq)]
enum InstallModalInputData {
    None,
    SortData(InstallModalSortData)
}
#[derive(Clone, PartialEq)]
struct InstallModalSortData {
    list: Vec<InstallModalListEntry>,
    current_path: String,
    selected_paths: Vec<String>
}
#[derive(Clone, PartialEq)]
struct InstallModalListEntry {
    name: String,
    path: String,
    folder: bool,
    selected: bool
}
#[derive(Clone)]
struct InstallModalInstallData {
    selected_paths: Vec<String>,
    copy_archive: bool,
    delete_archive: bool,
    name: String,
    version: String,
    category: String,
    specific_index_target: SpecificIndexTarget,
    specific_index: usize
}
#[derive(Clone, PartialEq)]
enum SpecificIndexTarget {
    Last,
    Replace,
    Before
}

impl SuperPatchApp {
    fn new(
        _cc: &eframe::CreationContext<'_>,
        orderdata: Value,
        vfsdata: VFSTree,
        vfscache: HashMap<String, VFSNode>,
        organizesort_list: Vec<OrganizeSortListEntry>,
        settings: Value,
        vfssort_list: Vec<VFSSortListEntry>,
        patchdata: Value,
    ) -> Self {
        Self {
            selected_tab: Tab::Organize,
            orderdata,
            vfsdata,
            vfscache,
            organizesort: OrganizeSort::PriorityAsc,
            organizesort_list,
            status: Arc::new(Mutex::new("Ready".to_string())),
            settings,
            vfssort: VFSSort {
                sort_type: VFSSortType::All,
                expanded: Vec::new(),
                query: String::new(),
            },
            vfssort_list,
            vfssort_list_refresh_requested: false,
            patchdata,
            livedata: LiveData {
                wine_modal_open: false,
                modded_exes_modal_open: false,
                import_modal_open: false,
                install_modal_open: false,
                install_modal_path: PathBuf::new(),
                install_modal_type: InstallModalType::None,
                install_modal_vfs_data: None,
                install_modal_input_data: InstallModalInputData::None,
                install_modal_install_data: None,
                install_modal_requested_install_index: 0,
                install_modal_requested_install_index_target: SpecificIndexTarget::Last,
                text_input_modal_open: false,
                text_input_modal_type: TextInputType::None,
                text_input_modal_value: String::new(),
                settingsedit_type: SettingsEditType::None,
                settingsedit_value: String::new(),
                organizeedit_type: OrganizeSortEditType::None,
                organizeedit_index: 0,
                organizeedit_value: String::new(),
            },
        }
    }
    //MARK: Unthreaded Actions
    fn refresh_all(&mut self) {
        (self.orderdata,self.vfsdata,self.vfscache,self.organizesort_list,self.vfssort_list,) = refresh_data(self.organizesort.clone(),self.vfssort.clone(),self.patchdata.clone(),);
    }
    fn update_all(&mut self) {
        (self.vfsdata, self.vfscache, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone(), self.vfscache.clone());
    }
    fn update_vfs_sort(&mut self) {
        self.vfssort_list = gen_vfs_sort_data(self.vfsdata.clone(),self.vfssort.clone(),self.patchdata.clone(),);
    }
    fn update_order_sort(&mut self) {
        self.organizesort_list = sort_organize_data(self.orderdata.clone(), self.organizesort.clone());
    }
    fn clear_organize_edit(&mut self) {
        self.livedata.organizeedit_value.clear();
        self.livedata.organizeedit_type = OrganizeSortEditType::None;
        self.livedata.organizeedit_index = 0;
    }
    fn clear_settings_edit(&mut self) {
        self.livedata.settingsedit_value.clear();
        self.livedata.settingsedit_type = SettingsEditType::None;
        
    }
    fn update_install_modal_sort_list(&mut self) {
        let vfs_data = self.livedata.install_modal_vfs_data.take().unwrap();
        let mut vfs_tree = match vfs_data {
            VFSNode::Dir(tree) => tree,
            _ => panic!()
        };
        let mut sort_list = Vec::<InstallModalListEntry>::new();
        let current_path = match &self.livedata.install_modal_input_data {
            InstallModalInputData::SortData(data) => {data.current_path.clone()},
            _ => {String::new()}
        };
        let selected_paths = match &self.livedata.install_modal_input_data {
            InstallModalInputData::SortData(data) => {data.selected_paths.clone()},
            _ => {vec![]}
        };
        if !current_path.is_empty() {
            for segment in current_path.split('/') {
                if segment.is_empty() {
                    continue;
                }
                match vfs_tree.remove(segment) {
                    Some(VFSNode::Dir(subtree)) => {
                        vfs_tree = subtree;
                    }
                    Some(VFSNode::File(_)) => {
                        panic!("path segment '{}' is a file, not a directory", segment);
                    }
                    None => {
                        panic!("path segment '{}' not found in VFS tree", segment);
                    }
                }
            }
        }
        for entry in vfs_tree {
            let mut new_entry = InstallModalListEntry {name: String::new(), path: String::new(), folder: false, selected: false};
            new_entry.folder = match entry.1 {
                VFSNode::Dir(_) => true,
                VFSNode::File(_) => false
            };
            new_entry.name = pathdiff(entry.0.as_str(), current_path.as_str());
            new_entry.path = if current_path.is_empty() {
                entry.0.clone()
            } else {
                format!("{}/{}", current_path, entry.0)
            };
            new_entry.selected = selected_paths.iter().any(|path_entry| path_entry == &new_entry.path);
            sort_list.push(new_entry);
        };
        self.livedata.install_modal_input_data = InstallModalInputData::SortData(InstallModalSortData {
            list: sort_list,
            current_path,
            selected_paths
        });
    }
    fn scan_mod_data(&mut self) {
        //Make VFS data for mod from archive.
        let ext = self.livedata.install_modal_path.extension().unwrap();
        let extension_type = if ext == OsStr::new("zip") {
            "zip"
        } else if ext == OsStr::new("rar") {
            "rar"
        } else if ext == OsStr::new("7z") || ext == OsStr::new("7zip") {
            "7z"
        } else {
            *self.status.lock().unwrap() = "Invalid file extension".to_string();
            self.livedata.install_modal_open = false;
            self.livedata.install_modal_path = PathBuf::new();
            self.livedata.install_modal_type = InstallModalType::None;
            return
        };
        let mut file = std::fs::File::open(&self.livedata.install_modal_path).unwrap();
        self.livedata.install_modal_vfs_data = match extension_type {
            "zip" => {

                let mut archive = zip::ZipArchive::new(file).unwrap();
                Some(vfs_scan_zip(&mut archive, self.livedata.install_modal_path.to_str().unwrap()).unwrap())
            },
            "rar" => {
                Some(vfs_scan_rar(&self.livedata.install_modal_path, self.livedata.install_modal_path.to_str().unwrap()))
            },
            "7z" => {
                Some(vfs_scan_7z(&mut file, self.livedata.install_modal_path.to_str().unwrap()).unwrap())
            },
            _ => panic!()
        };
    }
    fn check_mod_type(&mut self) {
        //TODO Mods: Detect selectors and wizards.
        //One root, skip

        //Multiple roots, selector

        //FOMOD, wizard / selector (https://nexus-mods.github.io/NexusMods.App/developers/misc/AboutFomod/)

        //BAIN, wizard / selector (https://wrye-bash.github.io/docs/Wrye%20Bash%20Technical%20Readme.html) (I've never seen this)

        //Basic installer
        self.livedata.install_modal_type = InstallModalType::Basic;
    }
    fn install_mod(&mut self) {
        let install_data = match &self.livedata.install_modal_install_data {
            Some(install_data) => install_data,
            _ => panic!()
        };
        let ext = self.livedata.install_modal_path.extension().unwrap();
        let extension_type = if ext == OsStr::new("zip") {
            "zip"
        } else if ext == OsStr::new("rar") {
            "rar"
        } else if ext == OsStr::new("7z") || ext == OsStr::new("7zip") {
            "7z"
        } else {
            panic!()
        };
        if !Path::exists(Path::new("mods")) {
            fs::create_dir("mods").expect("Failed to create configs directory")
        }
        let target_path = Path::new("mods").join(install_data.name.clone());
        let file = std::fs::File::open(&self.livedata.install_modal_path).unwrap();
        //If you were replacing the mod, delete the old mod data.
        if install_data.specific_index_target == SpecificIndexTarget::Replace {
            let replace_folder = self.orderdata.get(install_data.specific_index).unwrap().get("path").unwrap_or(&Value::Null);   
            if let Value::String(string) = replace_folder
            && !string.is_empty() {
                let replace_path = Path::new(string);
                if replace_path.exists() {
                    fs::remove_dir_all(replace_path).unwrap();
                }
            }
        }
        match extension_type {
            "zip" => {
                let mut archive = zip::ZipArchive::new(file).unwrap();
                extract_dirs_from_zip(&mut archive, install_data.selected_paths.clone(), &target_path).expect("Failed to extract directory from zip");
            },
            "rar" => {
                extract_dirs_from_rar(&self.livedata.install_modal_path, install_data.selected_paths.clone(), &target_path).expect("Failed to extract directory from rar");
            },
            "7z" => {
                extract_dirs_from_7z(file.try_clone().unwrap(), install_data.selected_paths.clone(), &target_path).expect("Failed to extract directory from 7z");
            },
            _ => panic!()
        }
        let mut archive_path = String::new();
        if install_data.copy_archive {
            if !Path::exists(Path::new("archives")) {
                fs::create_dir("archives").expect("Failed to create configs directory");
            }
            let src = Path::new(&self.livedata.install_modal_path);
            let file_name = src.file_name().expect("Archive path has no file name");
            let dst = Path::new("archives").join(file_name);
            if src != dst {
                // Try hardlink first (same filesystem, instant, no extra disk usage)
                match fs::hard_link(src, &dst) {
                    Ok(()) => {}
                    Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
                        // Different filesystem — fall back to a full copy
                        fs::copy(src, &dst).expect("Failed to copy archive to archives directory");
                    }
                    Err(e) => {
                        panic!("Failed to hardlink or copy archive: {}", e);
                    }
                }
                //If you were replacing the mod, (and it's in a different path), delete the old mod's archive.
                if install_data.specific_index_target == SpecificIndexTarget::Replace {
                    let old_archive = self.orderdata.get(install_data.specific_index).unwrap().get("archive").unwrap_or(&Value::Null);
                    if let Value::String(string) = old_archive
                    && !string.is_empty() {
                        let archive_path = Path::new(string);
                        if archive_path.exists() {
                            fs::remove_file(archive_path).expect("Archive could not be deleted.");
                        }
                    }
                }
            }
            archive_path = dst.to_string_lossy().to_string();
        }
        if install_data.delete_archive {
            fs::remove_file(self.livedata.install_modal_path.clone()).expect("Failed to delete archive");
        }
        let mut entry = json!({
            "enabled": true,
            "name": install_data.name,
            "path": target_path.to_string_lossy().to_string(),
            "version": install_data.version,
            "category": install_data.category,
            "archive": archive_path
        });
        match install_data.specific_index_target {
            SpecificIndexTarget::Replace => {
                *self.orderdata.as_array_mut().unwrap().get_mut(install_data.specific_index).unwrap() = entry;
            },
            SpecificIndexTarget::Before => {
                self.orderdata.as_array_mut().unwrap().insert(install_data.specific_index, entry);
            },
            SpecificIndexTarget::Last => {
                self.orderdata.as_array_mut().unwrap().push(entry);
            }
        }
        self.update_all();
        *self.status.lock().unwrap() = "Mod installed.".to_string();
    }
    //MARK: Threaded Actions
    fn launch(&mut self) {
        //Save settings if they were changed in the UI
        let edit_type = &self.livedata.settingsedit_type;
        match edit_type {
            SettingsEditType::GamePath => {
                self.settings["game_path"] =
                    Value::String(self.livedata.settingsedit_value.clone());
                save_settings(self.settings.clone());
                self.livedata.settingsedit_type = SettingsEditType::None;
            }
            SettingsEditType::GameCommand => {
                self.settings["game_command"] =
                    Value::String(self.livedata.settingsedit_value.clone());
                save_settings(self.settings.clone());
                self.livedata.settingsedit_type = SettingsEditType::None;
            }
            _ => {}
        }
        let settings = self.settings.clone();
        let vfsdata = self.vfsdata.clone();
        let patchdata = self.patchdata.clone();
        let status = Arc::clone(&self.status);
        thread::spawn(move || {
        //Stop launch if the game is already running.
        let mut system = System::new();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let game_running = system
            .processes_by_name(std::ffi::OsStr::new("Anomaly"))
            .next()
            .is_some();
        if game_running {
            *status.lock().unwrap() = "Game is already running. Please close it before launching again.".to_string();
            return;
        }
        //Stop launch if the game path or command is empty.
        if settings["game_path"].as_str().unwrap_or("").is_empty()
            || settings["game_command"]
                .as_str()
                .unwrap_or("")
                .is_empty()
        {
            *status.lock().unwrap() = "Game path or command is empty. Please set them before launching.".to_string();
            return;
        }
        *status.lock().unwrap() = "Checking game.".to_string();
        let mut vfsdata_active = vfsdata.clone();
        let game_path = settings["game_path"].as_str().unwrap_or("");
        let game_vfsdata = vfs_scan(game_path, game_path, VFSTree::new());
        vfsdata_active = merge_vfs_trees(vfsdata_active, game_vfsdata);
        *status.lock().unwrap() = "Moving files.".to_string();
        save_vfs_changes(Path::new(""));
        let real_vfs_path = realize_vfs_data(vfsdata_active, patchdata.clone());
        let game_command = settings["game_command"]
            .as_str()
            .unwrap_or("")
            .replace("%path%", real_vfs_path.to_str().unwrap_or(""));
        *status.lock().unwrap() = "Launching game.".to_string();
        //TODO Launch: Display error message if the command fails to launch the game.
        let _ = Command::new("sh")
            .arg("-c")
            .arg(game_command)
            .spawn();
        *status.lock().unwrap() = "Launched game.".to_string();
        });
    }
}

impl eframe::App for SuperPatchApp {
    //MARK: UI Begin
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.style_mut().interaction.selectable_labels = false;
        let saved_maximized = self.settings["window_maximized"].as_bool().unwrap_or(false);
        let viewport_maximized = ui
            .ctx()
            .input(|i| i.viewport().maximized)
            .unwrap_or(saved_maximized);
        if saved_maximized != viewport_maximized {
            self.settings["window_maximized"] = Value::Bool(viewport_maximized);
            save_settings(self.settings.clone());
        }
        if !viewport_maximized {
            let current_size = ui.ctx().viewport_rect().size();
            let saved_size = self.settings["window_size"]
                .as_array()
                .cloned()
                .unwrap_or_else(|| vec![Value::from(800.0), Value::from(600.0)]);
            let saved_size = egui::vec2(
                saved_size[0].as_f64().unwrap_or(800.0) as f32,
                saved_size[1].as_f64().unwrap_or(600.0) as f32,
            );
            if saved_size != current_size {
                self.settings["window_size"] = Value::Array(vec![
                    Value::from(current_size.x as f64),
                    Value::from(current_size.y as f64),
                ]);
                save_settings(self.settings.clone());
            }
        }
        //TODO Tools: Initial setup modal for first time users.
        //If you on windows, https://neacsu.net/posts/win_symlinks/
        if !self.settings["initialized"].as_bool().unwrap_or(false) {
            let initial_modal = Modal::new(Id::new("initial_modal")).show(ui.ctx(), |ui| {
                ui.heading("Initial Setup");
                ui.label("This feature is not yet implemented.");
            });
            if initial_modal.should_close() {
                self.settings["initialized"] = Value::Bool(true);
                save_settings(self.settings.clone());
            }
        }
        //MARK: Wine Modal
        //TODO Tools: Install / detect wine or proton and set up wineprefix for the game. (Linux only)
        //GET WINE
        //Getting wine: Is steam installed?
        //Yes: Is a version of proton above 10.0 installed?
        //Yes: Use that.
        //No: Install proton 10.0.
        //No: Is wine installed?
        //Yes: Use that.
        //No: Install wine.
        //Prefix setup:
        //Create new prefix.
        //Get winetricks
        //Install cmd d3dcompiler_47 d3dx10 d3dx11_43 d3dx9 dx8vb quartz vcrun2022 dxvk
        #[cfg(target_os = "linux")]
        {
            if self.livedata.wine_modal_open {
                let wine_modal = Modal::new(Id::new("wine_modal")).show(ui.ctx(), |ui| {
                    ui.heading("Wine Setup");
                    ui.label("This feature is not yet implemented.");
                });
                if wine_modal.should_close() {
                    self.livedata.wine_modal_open = false;
                }
            }
        }
        //MARK: Modded EXEs Modal
        //TODO Tools: Install latest modded EXEs.
        if self.livedata.modded_exes_modal_open {
            let modded_exes_modal = Modal::new(Id::new("modded_exes_modal")).show(ui.ctx(), |ui| {
                ui.heading("Install Modded EXEs");
                ui.label("This feature is not yet implemented.");
            });
            if modded_exes_modal.should_close() {
                self.livedata.modded_exes_modal_open = false;
            }
        }
        //MARK: Import Modal
        //TODO Tools: Import from MO2
        //Modal
        //MO2 Path
        //Instance Path
        //Start/Cancel
        if self.livedata.import_modal_open {
            let import_modal = Modal::new(Id::new("import_modal")).show(ui.ctx(), |ui| {
                ui.heading("Import from MO2");
                ui.label("This feature is not yet implemented.");
            });
            if import_modal.should_close() {
                self.livedata.import_modal_open = false;
            }
        }
        //MARK: Installation Modal
        if self.livedata.install_modal_open {
            let install_modal = Modal::new(Id::new("install_modal")).show(ui.ctx(), |ui| {
                ui.heading("Install Mod");
                if self.livedata.install_modal_path == PathBuf::new() && ui.button("Select Mod").clicked() {
                    let path = pick_mod_file();
                    if let Some(path_full) = path {
                        self.livedata.install_modal_path = path_full;
                    }
                } else {
                    if self.livedata.install_modal_vfs_data.is_none() {
                        self.scan_mod_data();
                    }
                    if self.livedata.install_modal_type == InstallModalType::None {
                        self.check_mod_type();
                    }
                    if self.livedata.install_modal_install_data.is_none() {
                        if self.livedata.install_modal_input_data == InstallModalInputData::None {
                            self.update_install_modal_sort_list();
                        }
                        let input_data = match &mut self.livedata.install_modal_input_data {
                            InstallModalInputData::SortData(sortdata) => sortdata,
                            _ => panic!()
                        };
                        let mut needs_update = false;
                        match self.livedata.install_modal_type {
                            //MARK: Basic Installation
                            InstallModalType::Basic => {
                                ui.label("Mod type not detected. Please manually select root(s).");
                                ui.label("Manual modification of mod folder after install is permitted in this scenario.");
                                ui.vertical(|ui| { 
                                    if input_data.current_path.is_empty() {
                                        if input_data.selected_paths.contains(&String::new()) {
                                            if ui.button("Deselect root as root").clicked() {
                                                input_data.selected_paths.retain_mut(|x| x != &String::new())
                                            }
                                        } else {
                                            if ui.button("Select root as root").clicked() {
                                                input_data.selected_paths.push(String::new())
                                            }
                                        }
                                    }
                                    if !input_data.current_path.is_empty() && ui.button("...").clicked() {
                                            input_data.current_path = Path::new(&input_data.current_path.clone()).parent().and_then(|p| p.to_str()).unwrap_or("").to_string();
                                            needs_update = true;
                                        }
                                    for entry in input_data.list.iter() {
                                        ui.horizontal(|ui| {
                                            let mut checked = entry.selected;
                                            if ui.checkbox(&mut checked, "").changed() {
                                                if checked {
                                                input_data.selected_paths.push(entry.path.clone());
                                                } else {
                                                input_data.selected_paths.retain_mut(|x| x != &entry.path.clone());
                                                }
                                                needs_update = true;
                                            }
                                            
                                            if entry.folder {
                                                if ui.button(entry.name.clone()).clicked() {
                                                    input_data.current_path = entry.path.clone();
                                                    needs_update = true;
                                                } 
                                            }
                                            else {
                                                ui.label(entry.name.clone());
                                            }
                                        });
                                    }
                                });
                                if ui.add_enabled(!input_data.selected_paths.is_empty(), egui::Button::new("Proceed to Installation")).clicked() {
                                    let (name, version) = parse_archive_name(self.livedata.install_modal_path.file_stem().unwrap().to_str().unwrap());
                                    let copy_archive = self.settings["copy_archive"].as_bool().unwrap_or(true);
                                    let delete_archive = self.settings["delete_archive"].as_bool().unwrap_or(true);
                                    self.livedata.install_modal_install_data = Some(InstallModalInstallData { selected_paths: input_data.selected_paths.clone(), copy_archive, delete_archive, name: name.clone(), version, category: String::new(), specific_index: self.livedata.install_modal_requested_install_index, specific_index_target: self.livedata.install_modal_requested_install_index_target.clone()})
                                }
                            },
                            //TODO Mods: Implement selectors and wizards
                            _ => {ui.close()}
                        }
                        if needs_update {
                            self.update_install_modal_sort_list();
                        }
                    } 
                    //MARK: Final Installation
                    else {
                        let mut install_data = match &mut self.livedata.install_modal_install_data {
                            Some(install_data) => install_data,
                            None => panic!()
                        };
                        ui.label("Selected Paths:");
                        for i in &install_data.selected_paths {
                            if i.is_empty() {
                                ui.label("[archive root]");
                            } else {
                                ui.label(i);
                            }
                        }
                        let mut copy_archive = self.settings["copy_archive"].as_bool().unwrap_or(true);
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut copy_archive, "").changed() {
                                self.settings["copy_archive"] = Value::Bool(copy_archive);
                                install_data.copy_archive = copy_archive;
                                save_settings(self.settings.clone());
                            }
                            ui.label("Copy Archive")
                        });
                        let mut delete_archive = self.settings["delete_archive"].as_bool().unwrap_or(true);
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut delete_archive, "").changed() {
                                self.settings["delete_archive"] = Value::Bool(delete_archive);
                                install_data.delete_archive = delete_archive;
                                save_settings(self.settings.clone());
                            }
                            ui.label("Delete Orginal Archive")
                        });
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut install_data.name);
                        ui.label("Version:");
                        ui.text_edit_singleline(&mut install_data.version);
                        ui.label("Category:");
                        ui.text_edit_singleline(&mut install_data.category);
                        if ui.button("Install Mod").clicked() {
                            self.install_mod();
                            ui.close();
                        }
                    }
                }
            });
            if install_modal.should_close() {
                self.livedata.install_modal_open = false;
                self.livedata.install_modal_path = PathBuf::new();
                self.livedata.install_modal_type = InstallModalType::None;
                self.livedata.install_modal_vfs_data = None;
                self.livedata.install_modal_input_data = InstallModalInputData::None;
                self.livedata.install_modal_install_data = None;
                self.livedata.install_modal_requested_install_index = 0;
                self.livedata.install_modal_requested_install_index_target = SpecificIndexTarget::Last;
            }
        }
        //MARK: Text input modal
        //TODO General: General purpose text input modal
        if self.livedata.text_input_modal_open {
            let text_input_modal = Modal::new(Id::new("text_input_modal")).show(ui.ctx(), |ui| {
                ui.heading("Text Input");
                ui.label("This feature is not yet implemented.");
            });
            if text_input_modal.should_close() {
                self.livedata.text_input_modal_open = false;
            }
        }
        //MARK: Menu Bar
        egui::Panel::top("top_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import").clicked() {
                        self.livedata.import_modal_open = true;
                    }
                    if ui.button("Add mod").clicked() {
                        let path = pick_mod_file();
                        if let Some(path_full) = path {
                            self.livedata.install_modal_open = true;
                            self.livedata.install_modal_path = path_full;
                            self.livedata.install_modal_requested_install_index = 0;
                            self.livedata.install_modal_requested_install_index_target = SpecificIndexTarget::Last;
                        }
                    }
                    if ui.button("Refresh").clicked() {
                        self.refresh_all()
                    }
                    if ui.button("Save VFS Changes").clicked() {
                        save_vfs_changes(Path::new(""));
                    }
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Tools", |ui| {
                    #[cfg(target_os = "linux")]
                    {
                        if ui.button("Setup Wine").clicked() {
                            self.livedata.wine_modal_open = true;
                        }
                        if ui.button("Install Modded EXEs").clicked() {
                            self.livedata.modded_exes_modal_open = true;
                        }
                    }
                });
            });
        });
        //MARK: Right Panel
        egui::Panel::right("right_panel").show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading("Superpatch");
                ui.label("v0.1.0");
                if ui.button("Launch").clicked() {
                    self.launch();
                }

                ui.label("Game Path:");
                let mut game_path_setting =
                    if self.livedata.settingsedit_type == SettingsEditType::GamePath {
                        self.livedata.settingsedit_value.clone()
                    } else {
                        self.settings["game_path"].as_str().unwrap_or("").to_string()
                    };
                let response = ui.text_edit_multiline(&mut game_path_setting);
                if response.changed() {
                    self.livedata.settingsedit_type = SettingsEditType::GamePath;
                    self.livedata.settingsedit_value = game_path_setting.clone();
                }
                if response.lost_focus() {
                    self.settings["game_path"] = Value::String(game_path_setting.to_string());
                    save_settings(self.settings.clone());
                    self.clear_settings_edit();
                }

                ui.label("Game Command:");
                let mut game_command_setting =
                    if self.livedata.settingsedit_type == SettingsEditType::GameCommand {
                        self.livedata.settingsedit_value.clone()
                    } else {
                        self.settings["game_command"].as_str().unwrap_or("").to_string()
                    };
                let response = ui.text_edit_multiline(&mut game_command_setting);
                if response.changed() {
                    self.livedata.settingsedit_type = SettingsEditType::GameCommand;
                    self.livedata.settingsedit_value = game_command_setting.clone();
                }
                if response.lost_focus() {
                    self.settings["game_command"] = Value::String(game_command_setting.to_string());
                    save_settings(self.settings.clone());
                    self.clear_settings_edit();
                }
            });
        });
        //MARK: Bottom Panel
        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(self.status.clone().lock().unwrap().as_str());
                match self.selected_tab {
                    Tab::Organize => {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |_ui| {
                            //TODO General: Search bar for Organize Tab
                        });
                    }
                    Tab::Files => {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            //TODO General: Search bar for Files Tab
                            let sort_text = match self.vfssort.sort_type {
                                VFSSortType::All => "Show All",
                                VFSSortType::Conflicts => "Show Conflicts",
                                VFSSortType::DLTXPatches => "Show DLTX Patches",
                            };
                            egui::ComboBox::from_label("Filter")
                                .selected_text(sort_text)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(self.vfssort.sort_type == VFSSortType::All,"Show All").clicked() {
                                        self.vfssort.sort_type = VFSSortType::All;
                                        self.update_vfs_sort();
                                    }
                                    if ui.selectable_label(self.vfssort.sort_type == VFSSortType::Conflicts, "Show Conflicts").clicked() {
                                        self.vfssort.sort_type = VFSSortType::Conflicts;
                                        self.update_vfs_sort();
                                    }
                                    if ui.selectable_label(self.vfssort.sort_type == VFSSortType::DLTXPatches, "Show DLTX Patches").clicked() {
                                        self.vfssort.sort_type = VFSSortType::DLTXPatches;
                                        self.update_vfs_sort();
                                    }
                                });
                        });
                    }
                    Tab::Patch => {}
                }
            });
        });
        //MARK: Top Selector
        egui::Panel::top("top_selector").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Organize, "Organize");
                ui.selectable_value(&mut self.selected_tab, Tab::Files, "Files");
                ui.selectable_value(&mut self.selected_tab, Tab::Patch, "Patch");
            });
        });
        //MARK: Patch Left Panel
        if self.selected_tab == Tab::Patch {
            egui::Panel::left("left_panel").show(ui, |_ui| {});
        }

        egui::CentralPanel::default().show(ui, |ui| {
            match self.selected_tab {
                //MARK: Configuration Page
                Tab::Organize => {
                    //TODO General: Fix table sizing issues
                    //Underfill: Calculate new size upon window resizing relative to previous size.
                    //Overflow: ???
                    let original_widths = self.settings["organize_widths"].as_array().cloned().unwrap_or_else(|| vec![Value::from(50.0), Value::from(200.0), Value::from(100.0), Value::from(100.0), Value::from(100.0)]);
                    TableBuilder::new(ui)
                    .sense(egui::Sense::click_and_drag())
                    .striped(true)
                    .auto_shrink(false)
                    .column(Column::initial(original_widths[0].as_f64().unwrap_or(50.0) as f32).resizable(true).at_least(50.0))
                    .column(Column::initial(original_widths[1].as_f64().unwrap_or(200.0) as f32).resizable(true).at_least(200.0).clip(true))
                    .column(Column::initial(original_widths[2].as_f64().unwrap_or(100.0) as f32).resizable(true).at_least(100.0).clip(true))
                    .column(Column::initial(original_widths[3].as_f64().unwrap_or(100.0) as f32).resizable(true).at_least(100.0).clip(true))
                    .column(Column::remainder().at_least(100.0).clip(true))
                    //MARK: Headers
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.label("Enabled"); });
                        header.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Name"); 
                                let sort_button = match self.organizesort {
                                    OrganizeSort::NameAsc => "⬆",
                                    OrganizeSort::NameDesc => "⬇",
                                    _ => "⬍",
                                };
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new(sort_button).frame(false)).clicked() {
                                        self.organizesort = match self.organizesort {
                                            OrganizeSort::NameAsc => OrganizeSort::NameDesc,
                                            OrganizeSort::NameDesc => OrganizeSort::NameAsc,
                                            _ => OrganizeSort::NameAsc,
                                        };
                                        self.update_order_sort();
                                    }
                                });
                            });
                        });
                        header.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Category");
                                let sort_button = match self.organizesort {
                                    OrganizeSort::CategoryAsc => "⬆",
                                    OrganizeSort::CategoryDesc => "⬇",
                                    _ => "⬍",
                                };
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new(sort_button).frame(false)).clicked() {
                                        self.organizesort = match self.organizesort {
                                            OrganizeSort::CategoryAsc => OrganizeSort::CategoryDesc,
                                            OrganizeSort::CategoryDesc => OrganizeSort::CategoryAsc,
                                            _ => OrganizeSort::CategoryAsc,
                                        };
                                        self.update_order_sort();
                                    }
                                });
                            });
                        });
                        header.col(|ui| { ui.label("Version"); });
                        header.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Priority");
                                let sort_button = match self.organizesort {
                                    OrganizeSort::PriorityAsc => "⬆",
                                    OrganizeSort::PriorityDesc => "⬇",
                                    _ => "⬍",
                                };
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new(sort_button).frame(false)).clicked() {
                                        self.organizesort = match self.organizesort {
                                            OrganizeSort::PriorityAsc => OrganizeSort::PriorityDesc,
                                            OrganizeSort::PriorityDesc => OrganizeSort::PriorityAsc,
                                            _ => OrganizeSort::PriorityAsc,
                                        };
                                        self.update_order_sort();
                                    }
                                });
                            });
                        });
                    })
                    //MARK: Body
                    .body(|mut body |{
                        let row_height = 20.0;
                        let num_rows = self.organizesort_list.len();
                        let ui_clone = body.ui_mut().ctx().clone();
                        let painter_clone = body.ui_mut().painter().clone();
                        let current_widths = body.widths();
                        if current_widths != original_widths.par_iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect::<Vec<f32>>() {
                            self.settings["organize_widths"] = Value::Array(current_widths.par_iter().map(|&w| Value::from(w as f64)).collect());
                            save_settings(self.settings.clone());
                        }
                        body.rows(row_height, num_rows, |mut row | {
                            let row_index = row.index();
                            let mod_entry = self.organizesort_list[row_index].clone();
                            let priority = mod_entry.priority;
                            let name = mod_entry.name.as_str();
                            let category = mod_entry.category.as_str();
                            let version = mod_entry.version.as_str();
                            let mut enabled = mod_entry.enabled;
                            row.col(|ui| {
                                ui.checkbox(&mut enabled, "").changed().then(|| {
                                    self.orderdata[priority as usize]["enabled"] = Value::Bool(enabled);
                                    self.update_all();
                                    *self.status.lock().unwrap() = if enabled { "Enabled mod.".to_string() } else { "Disabled mod.".to_string() };
                                });
                            });
                            //FIXME General: If you right click one of the labels it doesn't give you a right click menu.
                            for i in [("name", name, OrganizeSortEditType::Name), ("category", category, OrganizeSortEditType::Category), ("version", version, OrganizeSortEditType::Version)] {
                                row.col(|ui| {
                                    if self.livedata.organizeedit_type == i.2
                                    && self.livedata.organizeedit_index == priority as usize
                                    {
                                        let mut value = self.livedata.organizeedit_value.clone();
                                        let response = ui.add(egui::TextEdit::singleline(&mut value));
                                        if response.lost_focus() {
                                            self.orderdata[priority as usize][i.0] = Value::String(value.clone());
                                            self.update_all();
                                            self.clear_organize_edit();
                                        }
                                        if response.changed() {
                                            self.livedata.organizeedit_value = value;
                                        }
                                        response.request_focus();
                                    }
                                    else {
                                        let response = ui.add(egui::Label::new(i.1).sense(egui::Sense::click()));
                                        if response.double_clicked() {
                                            self.livedata.organizeedit_type = i.2;
                                            self.livedata.organizeedit_index = priority as usize;
                                            self.livedata.organizeedit_value = i.1.to_string();
                                            
                                        }
                                    }
                                });
                            }
                            row.col(|ui| {
                                ui.label(priority.to_string());
                            });
                            //MARK: Response
                            let response = row.response();
                            if self.organizesort == OrganizeSort::PriorityAsc || self.organizesort == OrganizeSort::PriorityDesc {
                                response.dnd_set_drag_payload(priority);
                            } else {
                                if response.drag_started() {
                                    *self.status.lock().unwrap() = "Drag and Drop is only available when sorting by Priority.".to_string();
                                }
                            }
                            if let (Some(pointer), Some(_hovered_payload)) = (
                                ui_clone.pointer_interact_pos(), row.response().dnd_hover_payload::<i64>(),
                            ) {
                                let rect = response.rect;
                                let stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
                                let mut insert_row_index = if pointer.y < rect.center().y {
                                    painter_clone.hline(rect.x_range(), rect.top(), stroke);
                                    row_index
                                } else {
                                    painter_clone.hline(rect.x_range(), rect.bottom(), stroke);
                                    row_index + 1
                                };

                                if let Some(dragged_payload) = row.response().dnd_release_payload::<i64>() {
                                    let dragged_index = *dragged_payload;
                                    if self.organizesort == OrganizeSort::PriorityDesc {
                                            insert_row_index = self.orderdata.as_array().cloned().unwrap_or_else(Vec::new).len() - insert_row_index;
                                    }
                                    if dragged_index != priority {
                                        let mut new_orderdata = self.orderdata.as_array().cloned().unwrap_or_else(Vec::new);

                                        let dragged_item = new_orderdata.remove(dragged_index as usize);
                                        let insert_index = if dragged_index < insert_row_index as i64 {
                                            insert_row_index - 1
                                        } else {
                                            insert_row_index
                                        };
                                        new_orderdata.insert(insert_index, dragged_item);
                                        self.orderdata = Value::Array(new_orderdata);
                                        self.update_all();
                                    }
                                }
                                //TODO Mods: Drag and drop mods
                            }
                            response.context_menu(|ui| {
                                if ui.button(if enabled { "Disable" } else { "Enable" }).clicked() {
                                    self.orderdata[priority as usize]["enabled"] = Value::Bool(!enabled);
                                    self.update_all();
                                    *self.status.lock().unwrap() = if enabled { "Disabled mod.".to_string() } else { "Enabled mod.".to_string() };
                                    ui.close();
                                }
                                if ui.button("Copy name").clicked() {
                                    ui.ctx().copy_text(name.to_owned());
                                    *self.status.lock().unwrap() = "Copied mod name to clipboard.".to_string();
                                    ui.close();
                                }
                                if ui.button("Rename mod").clicked() {
                                    self.livedata.organizeedit_type = OrganizeSortEditType::Name;
                                    self.livedata.organizeedit_index = priority as usize;
                                    self.livedata.organizeedit_value = name.to_string();
                                }
                                if ui.button("Edit Category").clicked() {
                                    self.livedata.organizeedit_type = OrganizeSortEditType::Category;
                                    self.livedata.organizeedit_index = priority as usize;
                                    self.livedata.organizeedit_value = category.to_string();
                                }
                                if ui.button("Change Version").clicked() {
                                    self.livedata.organizeedit_type = OrganizeSortEditType::Version;
                                    self.livedata.organizeedit_index = priority as usize;
                                    self.livedata.organizeedit_value = version.to_string();
                                }
                                if ui.button("Delete mod").clicked() {
                                    //TODO General: Warn user about deletion.
                                    if mod_entry.path.as_str() != "" {
                                        fs::remove_dir_all(mod_entry.path.as_str()).expect("Failed to delete mod directory");
                                    }
                                    let archive = self.orderdata.get(priority as usize).unwrap().get("archive").unwrap_or(&Value::Null);
                                    if let Value::String(string) = archive
                                    && !string.is_empty() {
                                        let archive_path = Path::new(string);
                                        if fs::exists(archive_path).unwrap() {
                                            fs::remove_file(archive_path).expect("Archive could not be deleted.");
                                        }
                                    }
                                    self.orderdata.as_array_mut().unwrap().remove(priority as usize);
                                    self.update_all();
                                    *self.status.lock().unwrap() = "Deleted mod.".to_string();
                                }
                                let archive_exists = matches!(self.orderdata.get(priority as usize).unwrap_or(&Value::Null).get("archive").unwrap_or(&Value::Null), Value::String(string) if !string.is_empty());
                                if ui.add_enabled(archive_exists, egui::Button::new("Reinstall mod")).clicked() {  
                                    let archive = self.orderdata.get(priority as usize).unwrap().get("archive").unwrap_or(&Value::Null);
                                    if let Value::String(string) = archive
                                    && !string.is_empty() {
                                        let archive_path = Path::new(string);
                                        if fs::exists(archive_path).unwrap() {
                                            self.livedata.install_modal_open = true;
                                            self.livedata.install_modal_path = archive_path.to_path_buf();
                                            self.livedata.install_modal_requested_install_index = priority as usize;
                                            self.livedata.install_modal_requested_install_index_target = SpecificIndexTarget::Replace;
                                        } else {
                                            self.orderdata.get_mut(priority as usize).unwrap().get_mut("archive").unwrap_or(&mut Value::Null).take();
                                            *self.status.lock().unwrap() = "Archive could not be found.".to_string();
                                        }
                                    }

                                    
                                }
                                if ui.button("Update mod").clicked() {
                                    //TODO Mods: Update mod (New install sequence with new file prompt, keep path.)
                                    //Specific index here, Replace
                                }
                                if ui.button("Open mod folder").clicked() {
                                    let mod_path = mod_entry.path.as_str();
                                    if !mod_path.is_empty() {
                                        if let Err(e) = open::that(mod_path) {
                                            *self.status.lock().unwrap() = format!("Failed to open mod folder: {}", e);
                                        }
                                    } else {
                                        *self.status.lock().unwrap() = "Mod path is empty.".to_string();
                                    }
                                    ui.close();
                                }
                            });
                        });
                    });
                    //MARK: Drag and Drop
                    if egui::DragAndDrop::has_payload_of_type::<i64>(ui.ctx()) {
                        let payload = *egui::DragAndDrop::payload::<i64>(ui.ctx()).unwrap();
                        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                            egui::Area::new(egui::Id::new("drag_and_drop_layer"))
                                .order(egui::Order::Tooltip)
                                .fixed_pos(pointer_pos + egui::vec2(16.0, 16.0))
                                .interactable(false)
                                .show(ui.ctx(), |ui| {
                                    ui.label(self.orderdata[payload as usize]["name"].as_str().unwrap_or(""));
                                });
                        }
                    }
                    if egui::DragAndDrop::has_any_payload(ui.ctx()) {
                        //TODO General: Scroll when touching edges of screen & dragging.
                    }
                }
                //MARK: Files Page
                Tab::Files => {
                    if self.vfssort_list_refresh_requested {
                        self.vfssort_list = gen_vfs_sort_data(self.vfsdata.clone(), self.vfssort.clone(), self.patchdata.clone());
                        self.vfssort_list_refresh_requested = false;
                    }
                    let original_widths = self.settings["files_widths"].as_array().cloned().unwrap_or_else(|| vec![Value::from(200.0), Value::from(100.0), Value::from(100.0)]);
                    //TODO General: Fix table sizing issues (See above.)
                    TableBuilder::new(ui)
                    .sense(egui::Sense::click())
                    .striped(true)
                    .auto_shrink(false)
                    .column(Column::initial(original_widths[0].as_f64().unwrap_or(200.0) as f32).resizable(true).at_least(200.0).clip(true))
                    .column(Column::initial(original_widths[1].as_f64().unwrap_or(100.0) as f32).resizable(true).at_least(100.0).clip(true))
                    .column(Column::initial(original_widths[2].as_f64().unwrap_or(100.0) as f32).resizable(true).at_least(100.0).clip(true))
                    .column(Column::remainder().at_least(100.0).clip(true))
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.label("Path"); });
                        header.col(|ui| { ui.label("File Type"); });
                        header.col(|ui| { ui.label("Conflicts"); });
                        header.col(|ui| { ui.label("DLTX Patches"); });
                    })
                    .body(|body |{
                        let row_height = 20.0;
                        let num_rows = self.vfssort_list.len();
                        let current_widths = body.widths();
                        if current_widths != original_widths.par_iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect::<Vec<f32>>() {
                            self.settings["files_widths"] = Value::Array(current_widths.par_iter().map(|&w| Value::from(w as f64)).collect());
                            save_settings(self.settings.clone());
                        }
                        body.rows(row_height, num_rows, |mut row | {
                            let row_index = row.index();
                            let file_entry = self.vfssort_list[row_index].clone();
                            let name = file_entry.name.as_str();
                            let down: usize = file_entry.down.try_into().unwrap_or(0);
                            let path = file_entry.path.as_str();
                            let file_type = file_entry.file_type.as_str();
                            let conflicts = file_entry.conflicts;
                            let dltx_patches = file_entry.dltx_patches;
                            let extended = file_entry.extended;
                            row.col(|ui| {
                                ui.horizontal(| ui| {
                                    ui.add_space(down as f32 * 22.0);
                                    if file_type == "folder" {
                                        if extended {
                                            ui.label("⬇");
                                        } else {
                                            ui.label("➡");
                                        }
                                    }
                                    ui.label(name);
                                });
                            });
                            row.col(|ui| { ui.label(file_type); });
                            row.col(|ui| {
                                if file_type != "folder" {
                                    ui.label(conflicts.to_string());
                                }
                            });
                            row.col(|ui| {
                                if file_type != "folder" {
                                    ui.label(dltx_patches.to_string());
                                }
                            });
                            if file_type == "folder"
                                && row.response().clicked() {
                                    if self.vfssort.expanded.contains(&path.to_string()) {
                                        self.vfssort.expanded.retain(|p| p != path);
                                        //Refreshing vfssort_list immediately will crash as the list gets smaller, but the UI is still referencing the larger size. Request refresh on the next frame.
                                        self.vfssort_list_refresh_requested = true;
                                    } else {
                                        self.vfssort.expanded.push(path.to_string());
                                        self.vfssort_list = gen_vfs_sort_data(self.vfsdata.clone(), self.vfssort.clone(), self.patchdata.clone());
                                    }
                                }
                        });
                    });

                }
                //MARK: Patch Page
                Tab::Patch => {
                    //TODO Patch: Patch page
                    ui.label("This feature is not yet implemented.");
                }
            }
        });
    }
}
//MARK: Main
fn main() {
    if !Path::exists(Path::new("configs")) {
        fs::create_dir("configs").expect("Failed to create configs directory");
    }
    let orderdata = read_order_data();
    let (vfsdata, vfscache) = gen_vfs_data(orderdata.clone(), HashMap::new());
    let organizesort_list = sort_organize_data(orderdata.clone(), OrganizeSort::PriorityAsc);
    let settings = read_settings();
    let patchdata = read_patch_data();
    let vfssort_list = gen_vfs_sort_data(
        vfsdata.clone(),
        VFSSort {
            sort_type: VFSSortType::All,
            expanded: Vec::new(),
            query: String::new(),
        },
        patchdata.clone(),
    );

    let viewport_size = settings["window_size"]
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![Value::from(800.0), Value::from(600.0)]);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::Vec2::new(
                viewport_size[0].as_f64().unwrap_or(800.0) as f32,
                viewport_size[1].as_f64().unwrap_or(600.0) as f32,
            ))
            .with_maximized(settings["window_maximized"].as_bool().unwrap_or(false)),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Superpatch",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(SuperPatchApp::new(
                cc,
                orderdata,
                vfsdata,
                vfscache,
                organizesort_list,
                settings,
                vfssort_list,
                patchdata,
            )))
        }),
    );
}

//MARK: Data Handling
fn read_order_data() -> Value {
    let orderdata_path = Path::new("configs/order.json");
    if !Path::exists(orderdata_path) {
        fs::write(orderdata_path, "[]").expect("Failed to create order.json");
    }
    let orderdata_str = fs::read_to_string(orderdata_path).expect("Failed to read order.json");
    let orderdata: Value =
        serde_json::from_str(&orderdata_str).expect("Failed to parse order.json");
    orderdata
}

fn refresh_data(
    organizesort: OrganizeSort,
    vfssort: VFSSort,
    patchdata: Value,
) -> (
    Value,
    VFSTree,
    HashMap<String, VFSNode>,
    Vec<OrganizeSortListEntry>,
    Vec<VFSSortListEntry>,
) {
    let orderdata = read_order_data();
    let (vfsdata, vfscache) = gen_vfs_data(orderdata.clone(), HashMap::new());
    let organizesort_list = sort_organize_data(orderdata.clone(), organizesort);
    let vfssort_list = gen_vfs_sort_data(vfsdata.clone(), vfssort, patchdata);
    (
        orderdata,
        vfsdata,
        vfscache,
        organizesort_list,
        vfssort_list,
    )
}

fn update_data(
    orderdata: Value,
    organizesort: OrganizeSort,
    vfssort: VFSSort,
    patchdata: Value,
    vfscache: HashMap<String, VFSNode>,
) -> (
    VFSTree,
    HashMap<String, VFSNode>,
    Vec<OrganizeSortListEntry>,
    Vec<VFSSortListEntry>,
) {
    let (vfsdata, vfscache) = gen_vfs_data(orderdata.clone(), vfscache);
    let text = serde_json::to_string_pretty(&orderdata).expect("Failed to serialize order.json");
    fs::write("configs/order.json", text).expect("Failed to write order.json");
    let organizesort_list = sort_organize_data(orderdata, organizesort);
    let vfssort_list = gen_vfs_sort_data(vfsdata.clone(), vfssort, patchdata);
    (vfsdata, vfscache, organizesort_list, vfssort_list)
}

fn read_settings() -> Value {
    let settings_path = Path::new("configs/settings.json");
    if !Path::exists(settings_path) {
        fs::write(settings_path, "{}").expect("Failed to create settings.json");
    }
    let settings_str = fs::read_to_string(settings_path).expect("Failed to read settings.json");
    let settings: Value =
        serde_json::from_str(&settings_str).expect("Failed to parse settings.json");
    settings
}

fn read_patch_data() -> Value {
    let patchdata_path = Path::new("configs/patch.json");
    if !Path::exists(patchdata_path) {
        fs::write(patchdata_path, "{}").expect("Failed to create patch.json");
    }
    let patchdata_str = fs::read_to_string(patchdata_path).expect("Failed to read patch.json");
    let patchdata: Value =
        serde_json::from_str(&patchdata_str).expect("Failed to parse patch.json");
    patchdata
}

fn save_settings(settings: Value) {
    let text = serde_json::to_string_pretty(&settings).expect("Failed to serialize settings.json");
    fs::write("configs/settings.json", text).expect("Failed to write settings.json");
}

//MARK: VFS Generation
fn gen_vfs_data(
    orderdata: Value,
    vfscache: HashMap<String, VFSNode>,
) -> (VFSTree, HashMap<String, VFSNode>) {
    let mut vfsdata = VFSTree::new();
    let mut vfscache = vfscache;
    for mod_entry in orderdata.as_array().unwrap().iter() {
        let name = mod_entry["name"].as_str().unwrap_or("");
        let path = mod_entry["path"].as_str().unwrap_or("");
        let enabled = mod_entry["enabled"].as_bool().unwrap_or(false);
        if !vfscache.contains_key(name) {
            if enabled {
                vfscache.insert(name.to_string(), vfs_scan(path, path, VFSTree::new()));
            }
        } else {
            if !enabled {
                vfscache.remove(name);
            }
        }
    }
    let stale_keys: Vec<String> = vfscache
        .keys()
        .filter(|key| {
            !orderdata
                .as_array()
                .unwrap()
                .par_iter()
                .any(|entry| entry["name"].as_str().unwrap_or("") == key.as_str())
        })
        .cloned()
        .collect();
    for name in stale_keys {
        vfscache.remove(&name);
    }
    for mod_entry in orderdata.as_array().unwrap().iter() {
        let name = mod_entry["name"].as_str().unwrap_or("");
        if let Some(vfsnode) = vfscache.get(name) {
            vfsdata = merge_vfs_trees(vfsdata, vfsnode.clone());
        }
    }
    (vfsdata, vfscache)
}

fn vfs_scan(path: &str, origin_path: &str, mut vfsdata: VFSTree) -> VFSNode {
    //TODO Patch: DLTX Patch support
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let relative_path = pathdiff(entry_path.to_string_lossy().as_ref(), origin_path);
            if entry_path.is_dir() {
                let sub_vfsdata = vfs_scan(
                    entry_path.to_str().unwrap_or(""),
                    origin_path,
                    VFSTree::new(),
                );
                vfsdata.insert(relative_path, sub_vfsdata);
            } else {
                let mut file_node = VFSNode::File(VFSFile {
                    paths: OrderedHashMap::new(),
                    dltx_patches: HashMap::new(),
                });
                if let VFSNode::File(ref mut file_data) = file_node {
                    file_data.paths.insert(
                        origin_path.to_string(),
                        entry_path.to_string_lossy().to_string(),
                    );
                }
                vfsdata.insert(relative_path, file_node);
            }
        }
    }
    VFSNode::Dir(vfsdata)
}

fn merge_vfs_trees(mut tree1: VFSTree, tree2: VFSNode) -> VFSTree {
    if let VFSNode::Dir(tree2_data) = tree2 {
        for (key, value) in tree2_data {
            if let Some(existing_node) = tree1.get_mut(&key) {
                if let VFSNode::Dir(existing_tree) = existing_node {
                    if let VFSNode::Dir(new_tree) = value {
                        *existing_tree =
                            merge_vfs_trees(existing_tree.clone(), VFSNode::Dir(new_tree));
                    }
                } else if let VFSNode::File(existing_file) = existing_node
                    && let VFSNode::File(new_file) = value {
                        existing_file.paths.extend(new_file.paths);
                        existing_file.dltx_patches.extend(new_file.dltx_patches);
                    }
            } else {
                tree1.insert(key, value);
            }
        }
    }
    tree1
}
//MARK: VFS Sorting
fn gen_vfs_sort_data(
    vfsdata: VFSTree,
    vfssort: VFSSort,
    patchdata: Value,
) -> Vec<VFSSortListEntry> {
    let mut vfssortlist =
        gen_vfs_sort_data_recursive(vfsdata, vfssort.clone(), String::new(), 0, patchdata);
    vfs_sort_data_prune_files(&mut vfssortlist, vfssort.clone());
    vfs_sort_data_prune_folders(&mut vfssortlist, vfssort);
    vfssortlist
}

fn gen_vfs_sort_data_recursive(
    vfsdata: VFSTree,
    vfssort: VFSSort,
    current_path: String,
    current_down: i64,
    patchdata: Value,
) -> Vec<VFSSortListEntry> {
    let mut vfssortlist = Vec::new();
    for (key, value) in vfsdata {
        let name = pathdiff(&key, &current_path);
        if let VFSNode::File(file) = value {
            let file_type = Path::new(&key)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("file")
                .to_string();
            let mut conflicts = file.paths.len().try_into().unwrap_or(0);
            if conflicts == 1 {
                conflicts = 0;
            }
            let dltx_patches = file.dltx_patches.len().try_into().unwrap_or(0);
            //TODO Patch: Check patchdata
            let dltx_patches_active = dltx_patches;
            let conflicts_active = conflicts;
            vfssortlist.push(VFSSortListEntry {
                path: key.clone(),
                name,
                down: current_down,
                extended: false,
                file_type,
                conflicts,
                conflicts_active,
                dltx_patches,
                dltx_patches_active,
            });
        } else {
            let extended = vfssort.expanded.contains(&key);
            vfssortlist.push(VFSSortListEntry {
                path: key.clone(),
                name,
                down: current_down,
                extended,
                file_type: "folder".into(),
                conflicts: 0,
                conflicts_active: 0,
                dltx_patches: 0,
                dltx_patches_active: 0,
            });
            if let VFSNode::Dir(children) = value {
                vfssortlist.extend(gen_vfs_sort_data_recursive(
                    children.clone(),
                    vfssort.clone(),
                    key.clone(),
                    current_down + 1,
                    patchdata.clone(),
                ));
            }
        }
    }
    vfssortlist
}

fn vfs_sort_data_prune_files(vfssortlist: &mut Vec<VFSSortListEntry>, vfssort: VFSSort) {
    for i in (0..vfssortlist.len()).rev() {
        let entry = &vfssortlist[i];
        if entry.file_type != "folder" {
            let mut remove = false;
            match vfssort.sort_type {
                VFSSortType::All => {}
                VFSSortType::Conflicts => {
                    if entry.conflicts == 0 {
                        remove = true;
                    }
                }
                VFSSortType::DLTXPatches => {
                    if entry.dltx_patches == 0 {
                        remove = true;
                    }
                }
            }
            if remove {
                vfssortlist.remove(i);
            }
        }
    }
}

fn vfs_sort_data_prune_folders(vfssortlist: &mut Vec<VFSSortListEntry>, vfssort: VFSSort) {
    for i in (0..vfssortlist.len()).rev() {
        let entry = vfssortlist[i].clone();
        if entry.file_type != "folder" {
            continue;
        }

        let has_children = vfssortlist
            .par_iter()
            .any(|child_entry| is_descendant_path(&child_entry.path, &entry.path));

        let remove_folder = match vfssort.sort_type {
            VFSSortType::All => false,
            VFSSortType::Conflicts | VFSSortType::DLTXPatches => !has_children,
        };

        if remove_folder {
            vfssortlist.remove(i);
        } else if !entry.extended {
            vfssortlist.retain(|child_entry| !is_descendant_path(&child_entry.path, &entry.path));
        }
    }
}
//MARK: VFS Realization
fn realize_vfs_data(vfsdata: VFSTree, patchdata: Value) -> PathBuf {
    //TODO Patch: Realize patchdata

    if fs::metadata(".vfs").is_ok() {
        fs::remove_dir_all(".vfs").expect("Failed to remove existing .vfs directory");
    }
    fs::create_dir(".vfs").expect("Failed to create .vfs directory");
    let vfs_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join(".vfs");
    link_vfs_data_recursive(vfsdata, &vfs_dir, patchdata);
    fetch_saved_vfs_changes(Path::new(""));
    vfs_dir
}

fn link_vfs_data_recursive(vfsdata: VFSTree, origin_path: &Path, patchdata: Value) {
    for (key, value) in vfsdata {
        let new_path = origin_path.join(&key);
        if let VFSNode::File(file) = value {
            let source_path = file.paths.iter().next_back().unwrap().1;
            link_that_file(&PathBuf::from(source_path), &new_path);
        } else if let VFSNode::Dir(children) = value {
            fs::create_dir_all(&new_path).expect("Failed to create directory");
            link_vfs_data_recursive(children, origin_path, patchdata.clone());
        }
    }
}

fn save_vfs_changes(mut current_dir: &Path) {
    if current_dir == "" {
        let vfs_dir = Path::new(".vfs");
        if !vfs_dir.exists() {
            return;
        } else {
            current_dir = vfs_dir;
        }
    }
    for entry in fs::read_dir(current_dir).expect("Failed to read current directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            save_vfs_changes(&path);
        } else if path.is_file()
            && !path.is_dir() {
                let metadata = fs::symlink_metadata(&path).expect("Failed to get file metadata");
                if !metadata.is_symlink() {
                    let relative_path = pathdiff(path.to_str().unwrap_or(""), ".vfs");
                    let saved_path = Path::new(".saved").join(&relative_path);
                    fs::create_dir_all(Path::new(".saved").join(&relative_path).parent().unwrap())
                        .expect("Failed to create .saved directory");
                    fs::copy(&path, &saved_path).expect("Failed to copy modified file to .saved");
                    fs::remove_file(&path).expect("Failed to remove modified file from .vfs");
                    link_that_file(&saved_path.canonicalize().unwrap(), &path);
                }
            }
    }
}

fn fetch_saved_vfs_changes(mut current_dir: &Path) {
    if current_dir == "" {
        let saved_dir = Path::new(".saved");
        if !saved_dir.exists() {
            return;
        } else {
            current_dir = saved_dir;
        }
    }
    for entry in fs::read_dir(current_dir).expect("Failed to read current directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            fetch_saved_vfs_changes(&path);
        } else if path.is_file() {
            let relative_path = pathdiff(path.to_str().unwrap_or(""), ".saved");
            let vfs_path = Path::new(".vfs").join(&relative_path);
            if vfs_path.exists() {
                fs::remove_file(&vfs_path).expect("Failed to remove existing file in .vfs");
            }
            fs::create_dir_all(vfs_path.parent().unwrap())
                .expect("Failed to create parent directories in .vfs");
            link_that_file(&path.canonicalize().unwrap(), &vfs_path);
        }
    }
}
//MARK: VFS Scan ZIP
//SLOP
fn vfs_scan_zip<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    origin_path: &str, // e.g. the zip file's own path, used as the "source" key
) -> Result<VFSNode, zip::result::ZipError> {
    let mut root = VFSTree::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;

        // `enclosed_name()` gives a sanitized relative Path (safe against zip-slip),
        // skip entries that don't resolve to a safe path.
        let Some(path) = file.enclosed_name() else {
            continue;
        };

        let is_dir = file.is_dir();
        drop(file); // release borrow on archive before recursing into by_index again

        insert_zip_entry(&mut root, &path, origin_path, is_dir);
    }

    Ok(VFSNode::Dir(root))
}
//SLOP
// Walk the path components of a single zip entry, creating/descending into
// VFSTree dirs as needed, then inserting the final file/dir node.\
fn insert_zip_entry(
    tree: &mut VFSTree,
    path: &std::path::Path,
    origin_path: &str,
    is_dir: bool,
) {
    let components: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    if components.is_empty() {
        return;
    }
    let mut current = tree;
    // descend through all but the last component, creating dirs as needed
    for comp in &components[..components.len() - 1] {
        let entry = current
            .entry(comp.clone())
            .or_insert_with(|| VFSNode::Dir(VFSTree::new()));
        match entry {
            VFSNode::Dir(sub) => current = sub,
            VFSNode::File(_) => {
                // conflict: a file exists where a dir is expected; skip or handle as needed
                return;
            }
        }
    }
    let last = &components[components.len() - 1];
    if is_dir {
        current
            .entry(last.clone())
            .or_insert_with(|| VFSNode::Dir(VFSTree::new()));
    } else {
        let relative_path = path.to_string_lossy().to_string();
        let node = current
            .entry(last.clone())
            .or_insert_with(|| {
                VFSNode::File(VFSFile {
                    paths: OrderedHashMap::new(),
                    dltx_patches: HashMap::new(),
                })
            });
        if let VFSNode::File(file_data) = node {
            file_data
                .paths
                .insert(origin_path.to_string(), relative_path);
        }
    }
}
//MARK: ZIP Extraction
//SLOP
//FIXME Mods: If a selected_path is in another selected_path, ignore it in the containing selected_path. 
fn extract_dirs_from_zip(
    archive: &mut ZipArchive<impl io::Read + io::Seek>,
    dir_list: Vec<String>, // e.g. "some/subdir/"
    output_root: &Path,
) -> zip::result::ZipResult<()> {
    for dir_prefix in dir_list {
        // Normalize prefix to always end with '/'
        let prefix = if dir_prefix.ends_with('/') || dir_prefix.is_empty() {
            dir_prefix.to_string()
        } else {
            format!("{}/", dir_prefix)
        };

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            // Only process entries under the target directory
            if !name.starts_with(&prefix) {
                continue;
            }
            // Compute the path relative to the extracted directory
            let relative = &name[prefix.len()..];
            if relative.is_empty() {
                continue; // this is the directory entry itself
            }
            let out_path = output_root.join(relative);
            if file.is_dir() {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut out_file = fs::File::create(&out_path)?;
                io::copy(&mut file, &mut out_file)?;
            }
            // Preserve Unix permissions if available
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }
    Ok(())
}

//MARK: VFS Scan 7Z
//SLOP
fn vfs_scan_7z<R: Read + Seek>(
    reader: &mut R,
    origin_path: &str,
) -> Result<VFSNode, sevenz_rust2::Error> {
    let archive = Archive::read(reader, &Password::empty())?;

    let mut tree: VFSTree = VFSTree::new();
    for entry in &archive.files {
        let name = entry.name.replace('\\', "/");
        let path = std::path::Path::new(&name);
        insert_zip_entry(&mut tree, path, origin_path, entry.is_directory);
    }

    Ok(VFSNode::Dir(tree))
}

//MARK: 7Z Extraction
//SLOP
//FIXME Mods: If a selected_path is in another selected_path, ignore it in the containing selected_path. 
fn extract_dirs_from_7z(
    mut reader: impl Read + std::io::Seek,
    dir_list: Vec<String>,
    output_root: &Path,
) -> Result<(), sevenz_rust2::Error> {
    for dir_prefix in dir_list {
        let prefix = if dir_prefix.is_empty() {
            String::new()
        } else if dir_prefix.ends_with('/') {
            dir_prefix.to_string()
        } else {
            format!("{}/", dir_prefix)
        };

        decompress_with_extract_fn(&mut reader, output_root, |entry: &ArchiveEntry, entry_reader: &mut dyn Read, _dest_path: &PathBuf| {
            let name = entry.name.replace('\\', "/");

            if !prefix.is_empty() && (!name.starts_with(&prefix) || name.len() == prefix.len()) {
                return Ok(false); // not under target dir, or is the dir entry itself
            }

            let relative = if prefix.is_empty() { name.as_str() } else { &name[prefix.len()..] };
            let out_path = output_root.join(relative);

            if entry.is_directory {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut out_file = fs::File::create(&out_path)?;
                io::copy(entry_reader, &mut out_file)?;
            }

            Ok(false) // we wrote it ourselves — tell the library not to also extract it
        })?;
    }
    Ok(())
}

//MARK: VFS Scan RAR
//SLOP
fn vfs_scan_rar(archive_path: &Path, origin_path: &str) -> VFSNode {
    let mut tree: VFSTree = VFSTree::new();

    for entry in unrar::Archive::new(archive_path).open_for_listing().unwrap() {
        let entry = entry.unwrap();
        let name = entry.filename.to_string_lossy().replace('\\', "/");
        insert_zip_entry(&mut tree, Path::new(&name), origin_path, entry.is_directory());
    }

    VFSNode::Dir(tree)
}

//MARK: RAR extraction
//SLOP
//FIXME Mods: If a selected_path is in another selected_path, ignore it in the containing selected_path. 
fn extract_dirs_from_rar(
    archive_path: &Path,
    dir_list: Vec<String>,
    output_root: &Path,
) -> unrar::error::UnrarResult<()> {
    for dir_prefix in dir_list {
        let prefix = if dir_prefix.is_empty() {
            String::new()
        } else if dir_prefix.ends_with('/') {
            dir_prefix.to_string()
        } else {
            format!("{}/", dir_prefix)
        };

        let mut archive = unrar::Archive::new(archive_path).open_for_processing()?;

        while let Some(cursor) = archive.read_header()? {
            let entry = cursor.entry();
            let name = entry.filename.to_string_lossy().replace('\\', "/");

            let matches = prefix.is_empty()
                || (name.starts_with(&prefix) && name.len() > prefix.len());

            if matches && !entry.is_directory() {
                let relative = if prefix.is_empty() { name.as_str() } else { &name[prefix.len()..] };
                let out_path = output_root.join(relative);
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).expect("Couldn't create directory");
                }

                let (data, rest) = cursor.read()?;
                fs::write(&out_path, data).expect("Couldn't write file");
                archive = rest;
            } else if matches && entry.is_directory() {
                let relative = if prefix.is_empty() { name.as_str() } else { &name[prefix.len()..] };
                if !relative.is_empty() {
                    fs::create_dir_all(output_root.join(relative)).expect("Couldn't create directory");
                }
                archive = cursor.skip()?;
            } else {
                archive = cursor.skip()?;
            }
        }
    }
    Ok(())
}

//MARK: Organize Sorting
fn sort_organize_data(orderdata: Value, organizesort: OrganizeSort) -> Vec<OrganizeSortListEntry> {
    let mut orderdata_array = orderdata.as_array().cloned().unwrap_or_else(Vec::new);
    let mut organizesortlist = Vec::new();
    //Add priority field
    for (i, item) in orderdata_array.iter_mut().enumerate() {
        organizesortlist.push(OrganizeSortListEntry {
            enabled: item
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            name: item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            category: item
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            version: item
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            priority: i as i64,
            path: item
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    match organizesort {
        OrganizeSort::NameAsc => organizesortlist.par_sort_by(|a, b| a.name.cmp(&b.name)),
        OrganizeSort::NameDesc => organizesortlist.par_sort_by(|a, b| b.name.cmp(&a.name)),
        OrganizeSort::CategoryAsc => organizesortlist.par_sort_by(|a, b| a.category.cmp(&b.category)),
        OrganizeSort::CategoryDesc => organizesortlist.par_sort_by(|a, b| b.category.cmp(&a.category)),
        OrganizeSort::PriorityAsc => organizesortlist.par_sort_by_key(|a| a.priority),
        OrganizeSort::PriorityDesc => organizesortlist.par_sort_by_key(|b| std::cmp::Reverse(b.priority)),
    }
    organizesortlist
}

//MARK: Tools
fn pathdiff(path: &str, reference: &str) -> String {
    let path = std::path::Path::new(path);
    let reference = std::path::Path::new(reference);
    path.strip_prefix(reference)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn is_descendant_path(path: &str, ancestor: &str) -> bool {
    let path = std::path::Path::new(path);
    let ancestor = std::path::Path::new(ancestor);
    path != ancestor
        && path
            .strip_prefix(ancestor)
            .map(|suffix| !suffix.as_os_str().is_empty())
            .unwrap_or(false)
}

fn link_that_file(source: &Path, destination: &Path) {
    #[cfg(target_family = "unix")]
    {
        if let Err(e) = std::os::unix::fs::symlink(source, destination) {
            eprintln!(
                "Failed to create symlink from {:?} to {:?}: {}",
                source, destination, e
            );
        }
    }
    #[cfg(target_family = "windows")]
    {
        if let Err(e) = std::os::windows::fs::symlink_file(source, destination) {
            eprintln!(
                "Failed to create symlink from {:?} to {:?}: {}",
                source, destination, e
            );
        }
    }
}

fn pick_mod_file() -> Option<PathBuf> {
    
    DialogBuilder::file()
        .set_location(&std::env::home_dir().unwrap_or(Path::new(".").to_path_buf()))
        .add_filter("All supported files", ["zip", "7z", "7zip", "rar"])
        .add_filter("Zip files", ["zip"])
        .add_filter("7z files", ["7z", "7zip"])
        .add_filter("Rar files", ["rar"])
        .add_filter("All files (Bad idea)", ["*"])
        .set_title("Select Mod Folder")
        .open_single_file()
        .show()
        .unwrap()
}

//SLOP
pub fn parse_archive_name(stem: &str) -> (String, String) {
    // Ordered from "definitely a version" to "eh, probably a version".
    // First match wins. Each returns the byte range to strip + the version text.
    let patterns = [
    r"(?i)v(?:er(?:sion)?)?[._-]?(\d+(?:[._]\d+)+[a-z]?)\b",
    r"(?i)\b(?:update|rev|build)[._-]?(\d+(?:[._]\d+)*)\b",
    r"(?:^|[^\d.])(\d+(?:\.\d+){2,}[a-z]?)\b",
    r"(?:^|[^\d.])(\d+\.\d+[a-z]?)\b",
    r"[._](\d+)$",
    ];

    for pat in &patterns {
        let re = Regex::new(pat).unwrap();
        if let Some(cap) = re.captures(stem) {
            let whole = cap.get(0).unwrap();
            let version = cap.get(1).unwrap().as_str().replace('_', ".");
            let name = format!("{}{}", &stem[..whole.start()], &stem[whole.end()..]);
            return (clean_name(&name), version)
        }
    }
    let date_str = Local::now().format("%-m/%-d/%y").to_string();
    (clean_name(stem), String::new())
}
//SLOP
fn clean_name(s: &str) -> String {
    let s = Regex::new(r"[._-]+").unwrap().replace_all(s, " ");
    Regex::new(r"\s+").unwrap().replace_all(s.trim(), " ").trim().to_string()
}