use std::{collections::{BTreeMap, HashMap}, path::PathBuf, process::Command};
use eframe::egui::{self};
use ordered_hash_map::OrderedHashMap;
use serde_json::{Value};
use std::fs;
use native_dialog::{DialogBuilder};
use std::path::Path;
use egui_extras::{TableBuilder, Column};
use sysinfo::System;

struct SuperPatchApp {
    selected_tab: Tab, 
    orderdata: Value, 
    vfsdata: VFSTree,
    vfscache: HashMap<String, VFSNode>,
    organizesort: OrganizeSort, 
    organizesort_list: Vec<OrganizeSortListEntry>,
    status: String,
    settings: Value,
    vfssort: VFSSort,
    vfssort_list: Vec<VFSSortListEntry>,
    vfssort_list_refresh_requested: bool,
    patchdata: Value,
    livedata: HashMap<String, DataOptions>
}
#[derive(PartialEq)]
enum Tab {
    Organize,
    Files,
    Patch
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
#[derive(Clone)]
struct SettingsEdit {
    edit_type: SettingsEditType,
    value: String
}
#[derive(PartialEq, Clone)]
enum SettingsEditType {
    None,
    GamePath,
    GameCommand
}
struct OrganizeSortEdit {
    edit_type: OrganizeSortEditType,
    index: usize,
    value: String
}
#[derive(PartialEq, Clone)]
enum OrganizeSortEditType {
    None,
    Name,
    Category,
    Version
}
#[derive(Clone)]
struct OrganizeSortListEntry {
    enabled: bool,
    name: String,
    category: String,
    version: String,
    priority: i64,
    path: String
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
    query: String
}
#[derive(PartialEq, Clone)]
enum VFSSortType {
    ShowAll,
    ShowConflicts,
    ShowDLTXPatches
}
type VFSTree = BTreeMap<String, VFSNode>;
#[derive(Clone, serde::Serialize)]
enum VFSNode {
    Dir(VFSTree),
    File(VFSFile)
}
#[derive(Clone)]
struct VFSFile {
    paths: OrderedHashMap<String, String>,
    dltx_patches: HashMap<String, String>
}

impl serde::Serialize for VFSFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("VFSFile", 2)?;
        state.serialize_field("paths", &self.paths.iter().collect::<Vec<(&String, &String)>>())?;
        state.serialize_field("dltx_patches", &self.dltx_patches)?;
        state.end()
    }
}
#[derive(PartialEq)]
enum DataOptions {
    SettingsEditType(SettingsEditType),
    OrganizeSortEditType(OrganizeSortEditType),
    usize(usize),
    String(String)
}

impl DataOptions {
    fn as_string(&self) -> String {
        match self {
            DataOptions::String(value) => value.clone(),
            _ => String::new(),
        }
    }
}

impl SuperPatchApp {
    fn new(
        _cc: &eframe::CreationContext<'_>, 
        orderdata: Value, 
        vfsdata: VFSTree, 
        vfscache: HashMap<String, VFSNode>, 
        organizesort_list: Vec<OrganizeSortListEntry>, 
        settings: Value, vfssort_list: Vec<VFSSortListEntry>, 
        patchdata: Value) -> Self {
            Self {
                selected_tab: Tab::Organize, 
                orderdata, 
                vfsdata, vfscache, 
                organizesort: OrganizeSort::PriorityAsc, 
                organizesort_list, 
                status: "Ready.".to_string(), 
                settings, 
                vfssort: VFSSort { sort_type: VFSSortType::ShowAll, expanded: Vec::new(), query: String::new() }, 
                vfssort_list, 
                vfssort_list_refresh_requested: false, 
                patchdata, 
                livedata: HashMap::new()
            }
    }
}

impl eframe::App for SuperPatchApp {
   fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.style_mut().interaction.selectable_labels = false;
        let saved_maximized = self.settings["window_maximized"].as_bool().unwrap_or(false);
        let viewport_maximized = ui.ctx().input(|i| i.viewport().maximized).unwrap_or(saved_maximized);
        if saved_maximized != viewport_maximized {
            self.settings["window_maximized"] = Value::Bool(viewport_maximized);
            save_settings(self.settings.clone());
        }
        if !viewport_maximized {
            let current_size = ui.ctx().viewport_rect().size();
            let saved_size = self.settings["window_size"].as_array().cloned().unwrap_or_else(|| vec![Value::from(800.0), Value::from(600.0)]);
            let saved_size = egui::vec2(saved_size[0].as_f64().unwrap_or(800.0) as f32, saved_size[1].as_f64().unwrap_or(600.0) as f32);
            if saved_size != current_size {
                self.settings["window_size"] = Value::Array(vec![Value::from(current_size.x as f64), Value::from(current_size.y as f64)]);
                save_settings(self.settings.clone());
            }
        }
        //MARK: Menu Bar
        egui::Panel::top("top_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import").clicked() {
                        //TODO Mods: Import from MO2
                        //Modal
                            //MO2 Path
                            //Instance Path
                            //Start/Cancel
                    }
                    if ui.button("Add mod").clicked() {
                        let path = DialogBuilder::file()
                            .set_location("~/Desktop")
                            .add_filter("Zip files", ["zip"])
                            .add_filter("7z files", ["7z", "7zip"])
                            .add_filter("Rar files", ["rar"])
                            .add_filter("All files", ["*"])
                            .set_title("Select Mod Folder")
                            .open_single_file()
                            .show()
                            .unwrap();
                        if path.is_some() {
                           self.orderdata.as_array_mut().unwrap().push(install_mod(path.as_ref().unwrap()));
                           (self.vfsdata, self.vfscache, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone(), self.vfscache.clone());
                        }
                    }
                    if ui.button("Refresh").clicked() {
                        (self.orderdata, self.vfsdata, self.vfscache, self.organizesort_list, self.vfssort_list) = refresh_data(self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone());       
                    }
                    if ui.button("Save VFS Changes").clicked() {
                        save_vfs_changes();
                    }
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Tools", |ui| {
                    #[cfg(target_os = "linux")]
                    {
                    if ui.button("Setup Wine").clicked() {
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
                        let wine_path = "/home/charlie/.steam/steam/steamapps/common/Proton 10.0/files/bin/wine64";
                    }
                    if ui.button("Install Modded EXEs").clicked() {
                        //TODO Tools: Install latest modded EXEs.
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
                    //Stop launch if the game is already running.
                    //HACK Launch: Is it always that?
                    let game_running = System::new_all().processes_by_name(std::ffi::OsStr::new("Anomaly")).next().is_some();
                    if game_running {
                        self.status = "Game is already running. Please close it before launching again.".to_string();
                        return;
                    }
                    //Stop launch if the game path or command is empty.
                    if self.settings["game_path"].as_str().unwrap_or("").is_empty() || self.settings["game_command"].as_str().unwrap_or("").is_empty() {
                        self.status = "Game path or command is empty. Please set them before launching.".to_string();
                        return;
                    }
                    //Save settings if they were changed in the UI
                    let edit_type = self.livedata.get("settingsedit_type");
                    match edit_type {
                        Some(DataOptions::SettingsEditType(SettingsEditType::GamePath)) => {
                            self.settings["game_path"] = Value::String(self.livedata.get("settingsedit_value").unwrap_or(&DataOptions::String(String::new())).as_string());
                            save_settings(self.settings.clone());
                            self.livedata.insert("settingsedit_type".to_string(), DataOptions::SettingsEditType(SettingsEditType::None));
                        }
                        Some(DataOptions::SettingsEditType(SettingsEditType::GameCommand)) => {
                            self.settings["game_command"] = Value::String(self.livedata.get("settingsedit_value").unwrap_or(&DataOptions::String(String::new())).as_string());
                            save_settings(self.settings.clone());
                            self.livedata.insert("settingsedit_type".to_string(), DataOptions::SettingsEditType(SettingsEditType::None));
                        }
                        None => {}
                        Some(_) => {}
                    }
                    let mut vfsdata_active = self.vfsdata.clone();
                    let game_path = self.settings["game_path"].as_str().unwrap_or("");
                    let game_vfsdata = vfs_scan(game_path, game_path, VFSTree::new());
                    vfsdata_active = merge_vfs_trees(vfsdata_active, game_vfsdata);
                    save_vfs_changes();
                    let real_vfs_path = realize_vfs_data(vfsdata_active, self.patchdata.clone());
                    let game_command = self.settings["game_command"].as_str().unwrap_or("").replace("%path%", real_vfs_path.to_str().unwrap_or(""));
                    println!("Launching game with command: {}", game_command);
                    //TODO Launch: Display error message if the command fails to launch the game.
                    Command::new("sh")
                        .arg("-c")
                        .arg(game_command)
                        .spawn()
                        .expect("Failed to launch game");
                    self.status = "Launched game.".to_string();
                }

                ui.label("Game Path:");
                let mut game_path_setting = if matches!(self.livedata.get("settingsedit_type"), Some(DataOptions::SettingsEditType(SettingsEditType::GamePath))) {
                    self.livedata.get("settingsedit_value").map(DataOptions::as_string).unwrap_or_else(|| "".into())
                } else {
                    self.settings["game_path"].as_str().unwrap_or("").to_string()
                };
                let response = ui.text_edit_multiline(&mut game_path_setting);
                if response.changed() {
                    self.livedata.insert("settingsedit_type".to_string(), DataOptions::SettingsEditType(SettingsEditType::GamePath));
                    self.livedata.insert("settingsedit_value".to_string(), DataOptions::String(game_path_setting.to_string()));
                }
                if response.lost_focus() {
                    self.settings["game_path"] = Value::String(game_path_setting.to_string());
                    save_settings(self.settings.clone());
                    self.livedata.insert("settingsedit_type".to_string(), DataOptions::SettingsEditType(SettingsEditType::None));
                }

                ui.label("Game Command:");
                let mut game_command_setting = if matches!(self.livedata.get("settingsedit_type"), Some(DataOptions::SettingsEditType(SettingsEditType::GameCommand))) {
                    self.livedata.get("settingsedit_value").map(DataOptions::as_string).unwrap_or_else(|| "".into())
                } else {
                    self.settings["game_command"].as_str().unwrap_or("").to_string()
                };
                let response = ui.text_edit_multiline(&mut game_command_setting);
                if response.changed() {
                    self.livedata.insert("settingsedit_type".to_string(), DataOptions::SettingsEditType(SettingsEditType::GameCommand));
                    self.livedata.insert("settingsedit_value".to_string(), DataOptions::String(game_command_setting.to_string()));
                }
                if response.lost_focus() {
                    self.settings["game_command"] = Value::String(game_command_setting.to_string());
                    save_settings(self.settings.clone());
                    self.livedata.insert("settingsedit_type".to_string(), DataOptions::SettingsEditType(SettingsEditType::None));
                }
            });
        });
        //MARK: Bottom Panel
        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(self.status.clone());
                match self.selected_tab {
                    Tab::Organize => {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            //TODO General: Search bar for organize Tab
                        });
                    }
                    Tab::Files => {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            //TODO General: Search bar for files Tab
                            let sort_text = match self.vfssort.sort_type {
                                VFSSortType::ShowAll => "Show All",
                                VFSSortType::ShowConflicts => "Show Conflicts",
                                VFSSortType::ShowDLTXPatches => "Show DLTX Patches",
                            };
                            egui::ComboBox::from_label("Filter")
                                .selected_text(sort_text)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(self.vfssort.sort_type == VFSSortType::ShowAll, "Show All").clicked() {
                                        self.vfssort.sort_type = VFSSortType::ShowAll;
                                        self.vfssort_list = gen_vfs_sort_data(self.vfsdata.clone(), self.vfssort.clone(), self.patchdata.clone());
                                    }
                                    if ui.selectable_label(self.vfssort.sort_type == VFSSortType::ShowConflicts, "Show Conflicts").clicked() {
                                        self.vfssort.sort_type = VFSSortType::ShowConflicts;
                                        self.vfssort_list = gen_vfs_sort_data(self.vfsdata.clone(), self.vfssort.clone(), self.patchdata.clone());
                                    }
                                    if ui.selectable_label(self.vfssort.sort_type == VFSSortType::ShowDLTXPatches, "Show DLTX Patches").clicked() {
                                        self.vfssort.sort_type = VFSSortType::ShowDLTXPatches;
                                        self.vfssort_list = gen_vfs_sort_data(self.vfsdata.clone(), self.vfssort.clone(), self.patchdata.clone());
                                    }
                                });
                        });
                    }
                    Tab::Patch => {}
                }
            });
        });
        //MARK: Top Selector
        egui::Panel::top("top_selector")   .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Organize, "Organize");
                ui.selectable_value(&mut self.selected_tab, Tab::Files, "Files");
                ui.selectable_value(&mut self.selected_tab, Tab::Patch, "Patch");
            });
        });
        //MARK: Patch Left Panel
        if self.selected_tab == Tab::Patch {
            egui::Panel::left("left_panel").show(ui, |ui| {

            });
        }

        egui::CentralPanel::default().show(ui, |ui| {
            match self.selected_tab {
                //MARK: Configuration Page
                Tab::Organize => {
                    //TODO General: fix table overflow or underfill
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
                                        self.organizesort_list = sort_organize_data(self.orderdata.clone(), self.organizesort.clone());
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
                                        self.organizesort_list = sort_organize_data(self.orderdata.clone(), self.organizesort.clone());
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
                                        self.organizesort_list = sort_organize_data(self.orderdata.clone(), self.organizesort.clone());
                                    }
                                });
                            });
                        });
                    })
                    .body(|mut body |{
                        let row_height = 20.0;
                        let num_rows = self.organizesort_list.len();
                        let ui_clone = body.ui_mut().ctx().clone();
                        let painter_clone = body.ui_mut().painter().clone();    
                        let current_widths = body.widths();
                        if current_widths != original_widths.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect::<Vec<f32>>() {
                            self.settings["organize_widths"] = Value::Array(current_widths.iter().map(|&w| Value::from(w as f64)).collect());
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
                                    self.orderdata[row_index]["enabled"] = Value::Bool(enabled);
                                    (self.vfsdata, self.vfscache, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone(), self.vfscache.clone());
                                    self.status = if enabled { "Enabled mod.".to_string() } else { "Disabled mod.".to_string() };
                                });
                            });
                            row.col(|ui| {
                                if matches!(self.livedata.get("organizeedit_type"), Some(DataOptions::OrganizeSortEditType(OrganizeSortEditType::Name)))
                                    && matches!(self.livedata.get("organizesort_edit_index"), Some(DataOptions::usize(row_index)))
                                {
                                    let response = ui.add(egui::TextEdit::singleline(&mut self.livedata.get("organizesort_edit_value").unwrap_or(&DataOptions::String(String::new())).as_string()));
                                    if response.lost_focus() {
                                        self.orderdata[priority as usize]["name"] = Value::String(self.livedata.get("organizesort_edit_value").unwrap_or(&DataOptions::String(String::new())).as_string());
                                        (self.vfsdata, self.vfscache, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone(), self.vfscache.clone());
                                        self.livedata.remove("organizeedit_type");
                                        self.livedata.remove("organizesort_edit_index");
                                        self.livedata.remove("organizesort_edit_value");
                                    }
                                    response.request_focus();
                                } 
                                else {
                                    let response = ui.add(egui::Label::new(name).sense(egui::Sense::click()));
                                    if response.double_clicked() {
                                        self.livedata.insert("organizeedit_type".to_string(), DataOptions::OrganizeSortEditType(OrganizeSortEditType::Name));
                                        self.livedata.insert("organizesort_edit_index".to_string(), DataOptions::usize(row_index));
                                        self.livedata.insert("organizesort_edit_value".to_string(), DataOptions::String(name.to_string()));
                                    }
                                }
                                
                            });
                            row.col(|ui| {
                                if matches!(self.livedata.get("organizeedit_type"), Some(DataOptions::OrganizeSortEditType(OrganizeSortEditType::Category)))
                                    && matches!(self.livedata.get("organizesort_edit_index"), Some(DataOptions::usize(row_index)))
                                {
                                    let response = ui.add(egui::TextEdit::singleline(&mut self.livedata.get("organizesort_edit_value").unwrap_or(&DataOptions::String(String::new())).as_string()));
                                    if response.lost_focus() {
                                        self.orderdata[priority as usize]["category"] = Value::String(self.livedata.get("organizesort_edit_value").unwrap_or(&DataOptions::String(String::new())).as_string());
                                        (self.vfsdata, self.vfscache, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone(), self.vfscache.clone());
                                        self.livedata.remove("organizeedit_type");
                                        self.livedata.remove("organizesort_edit_index");
                                        self.livedata.remove("organizesort_edit_value");
                                    }
                                    response.request_focus();
                                } 
                                else {
                                    let response = ui.add(egui::Label::new(category).sense(egui::Sense::click()));
                                    if response.double_clicked() {
                                        self.livedata.insert("organizeedit_type".to_string(), DataOptions::OrganizeSortEditType(OrganizeSortEditType::Category));
                                        self.livedata.insert("organizesort_edit_index".to_string(), DataOptions::usize(row_index));
                                        self.livedata.insert("organizesort_edit_value".to_string(), DataOptions::String(category.to_string()));
                                    }
                                }
                            });
                            row.col(|ui| {
                                if matches!(self.livedata.get("organizeedit_type"), Some(DataOptions::OrganizeSortEditType(OrganizeSortEditType::Version)))
                                    && matches!(self.livedata.get("organizesort_edit_index"), Some(DataOptions::usize(row_index)))
                                {
                                    let response = ui.add(egui::TextEdit::singleline(&mut self.livedata.get("organizesort_edit_value").unwrap_or(&DataOptions::String(String::new())).as_string()));
                                    if response.lost_focus() {
                                        self.orderdata[priority as usize]["version"] = Value::String(self.livedata.get("organizesort_edit_value").unwrap_or(&DataOptions::String(String::new())).as_string());
                                        (self.vfsdata, self.vfscache, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone(), self.vfscache.clone());
                                        self.livedata.remove("organizeedit_type");
                                        self.livedata.remove("organizesort_edit_index");
                                        self.livedata.remove("organizesort_edit_value");
                                    }
                                    response.request_focus();
                                } 
                                else {
                                    let response = ui.add(egui::Label::new(version).sense(egui::Sense::click()));
                                    if response.double_clicked() {
                                        self.livedata.insert("organizeedit_type".to_string(), DataOptions::OrganizeSortEditType(OrganizeSortEditType::Version));
                                        self.livedata.insert("organizesort_edit_index".to_string(), DataOptions::usize(row_index));
                                        self.livedata.insert("organizesort_edit_value".to_string(), DataOptions::String(version.to_string()));
                                    }
                                }
                            });
                            row.col(|ui| {
                                ui.label(priority.to_string()); 
                            }); 
                            let response = row.response();
                            if self.organizesort == OrganizeSort::PriorityAsc || self.organizesort == OrganizeSort::PriorityDesc {
                                response.dnd_set_drag_payload(priority); 
                            } else {
                                if response.drag_started() {
                                    self.status = "Drag and Drop is only available when sorting by Priority.".to_string();
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
                                    let dragged_index = *Some(dragged_payload).unwrap();
                                    if self.organizesort == OrganizeSort::PriorityDesc {
                                            insert_row_index = self.orderdata.as_array().cloned().unwrap_or_else(Vec::new).len() as usize - insert_row_index;
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
                                        (self.vfsdata, self.vfscache, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone(), self.vfscache.clone());
                                    }
                                }
                            }
                            response.context_menu(|ui| {
                                if ui.button(if enabled { "Disable" } else { "Enable" }).clicked() {
                                    self.orderdata[row_index]["enabled"] = Value::Bool(!enabled);
                                    (self.vfsdata, self.vfscache, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone(), self.vfscache.clone());
                                    self.status = if enabled { "Disabled mod.".to_string() } else { "Enabled mod.".to_string() };
                                    ui.close();
                                }
                                if ui.button("Copy name").clicked() {
                                    ui.ctx().copy_text(name.to_owned());
                                    self.status = "Copied mod name to clipboard.".to_string();
                                    ui.close();
                                }
                                if ui.button("Rename mod").clicked() {
                                    self.livedata.insert("organizeedit_type".to_string(), DataOptions::OrganizeSortEditType(OrganizeSortEditType::Name));
                                    self.livedata.insert("organizesort_edit_index".to_string(), DataOptions::usize(row_index));
                                    self.livedata.insert("organizesort_edit_value".to_string(), DataOptions::String(name.to_string()));
                                }
                                if ui.button("Edit Category").clicked() {
                                    self.livedata.insert("organizeedit_type".to_string(), DataOptions::OrganizeSortEditType(OrganizeSortEditType::Category));
                                    self.livedata.insert("organizesort_edit_index".to_string(), DataOptions::usize(row_index));
                                    self.livedata.insert("organizesort_edit_value".to_string(), DataOptions::String(category.to_string()));
                                }
                                if ui.button("Change Version").clicked() {
                                    self.livedata.insert("organizeedit_type".to_string(), DataOptions::OrganizeSortEditType(OrganizeSortEditType::Version));
                                    self.livedata.insert("organizesort_edit_index".to_string(), DataOptions::usize(row_index));
                                    self.livedata.insert("organizesort_edit_value".to_string(), DataOptions::String(version.to_string()));
                                }
                                if ui.button("Delete mod").clicked() {
                                    if mod_entry.path.as_str() != "" {
                                    fs::remove_dir_all(mod_entry.path.as_str()).expect("Failed to delete mod directory");
                                    }
                                    self.orderdata.as_array_mut().unwrap().remove(row_index);
                                    (self.vfsdata, self.vfscache, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone(), self.vfscache.clone());
                                    self.status = "Deleted mod.".to_string();
                                    ui.close();
                                }
                                if ui.button("Reinstall mod").clicked() {  
                                    //TODO Mods: Reinstall mod (Store in order.json and re-run installation)
                                    //Also, disable this button if the path is empty or the file doesn't exist.
                                }
                                if ui.button("Update mod").clicked() {
                                    //TODO Mods: Update mod (New install sequence with new file prompt, keep path.)
                                }
                                if ui.button("Rename mod folder").clicked() {
                                    //TODO Mods: Rename mod folder (update order.json)
                                    //TODO Patch: Update patch.json with new path
                                }
                                if ui.button("Open mod folder").clicked() {
                                    let mod_path = mod_entry.path.as_str();
                                    if !mod_path.is_empty() {
                                        if let Err(e) = open::that(mod_path) {
                                            self.status = format!("Failed to open mod folder: {}", e);
                                        }
                                    } else {
                                        self.status = "Mod path is empty.".to_string();
                                    }
                                    ui.close();
                                }
                            });
                        });
                    });
                    // Drag and Drop display
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
                        //TODO General: Scroll when you reach the bottom or top of the screen while dragging
                    }
                }
                //MARK: Files Page
                Tab::Files => {
                    if self.vfssort_list_refresh_requested {
                        self.vfssort_list = gen_vfs_sort_data(self.vfsdata.clone(), self.vfssort.clone(), self.patchdata.clone());
                        self.vfssort_list_refresh_requested = false;
                    }
                    let original_widths = self.settings["files_widths"].as_array().cloned().unwrap_or_else(|| vec![Value::from(200.0), Value::from(100.0), Value::from(100.0)]);
                    //TODO General: fix table overflow or underfill
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
                        if current_widths != original_widths.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect::<Vec<f32>>() {
                            self.settings["files_widths"] = Value::Array(current_widths.iter().map(|&w| Value::from(w as f64)).collect());
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
                            if file_type == "folder" {
                                if row.response().clicked() {
                                    if self.vfssort.expanded.contains(&path.to_string()) {
                                        self.vfssort.expanded.retain(|p| p != path);
                                        //Refreshing vfssort_list immediately will crash as the list gets smaller, but the UI is still referencing the larger size. Request refresh on the next frame.
                                        self.vfssort_list_refresh_requested = true;
                                    } else {
                                        self.vfssort.expanded.push(path.to_string());
                                        self.vfssort_list = gen_vfs_sort_data(self.vfsdata.clone(), self.vfssort.clone(), self.patchdata.clone());
                                    }
                                }
                            }
                        });
                    });

                }
                //MARK: Patch Page
                Tab::Patch => {
                    //TODO Patch: Patch page
                }
            }
        });
   }
}
//MARK: Main
fn main(){
    if !Path::exists(Path::new("configs")) {
        fs::create_dir("configs").expect("Failed to create configs directory");
    }
    let orderdata = read_order_data();
    let (vfsdata, vfscache) = gen_vfs_data(orderdata.clone(), HashMap::new());
    let organizesort_list = sort_organize_data(orderdata.clone(), OrganizeSort::PriorityAsc);
    let settings = read_settings();
    let patchdata = read_patch_data();
    let vfssort_list = gen_vfs_sort_data(vfsdata.clone(), VFSSort { sort_type: VFSSortType::ShowAll, expanded: Vec::new(), query: String::new() }, patchdata.clone());
    

    let viewport_size = settings["window_size"].as_array().cloned().unwrap_or_else(|| vec![Value::from(800.0), Value::from(600.0)]);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::Vec2::new(viewport_size[0].as_f64().unwrap_or(800.0) as f32, viewport_size[1].as_f64().unwrap_or(600.0) as f32))
            .with_maximized(settings["window_maximized"].as_bool().unwrap_or(false)),
        ..Default::default()
    };
    let _ = eframe::run_native("Superpatch", native_options, Box::new(|cc| {Ok(Box::new(SuperPatchApp::new(cc, orderdata, vfsdata, vfscache, organizesort_list, settings, vfssort_list, patchdata)))}));
}

//MARK: Data Handling
fn read_order_data() -> Value {
    let orderdata_path = Path::new("configs/order.json");
    if !Path::exists(orderdata_path) {
        fs::write(orderdata_path, "[]").expect("Failed to create order.json");
    }
    let orderdata_str = fs::read_to_string(orderdata_path).expect("Failed to read order.json");
    let orderdata: Value = serde_json::from_str(&orderdata_str).expect("Failed to parse order.json");
    orderdata
}

fn gen_vfs_data(orderdata: Value, vfscache: HashMap<String, VFSNode>) -> (VFSTree, HashMap<String, VFSNode>) {
    let mut vfsdata = VFSTree::new();
    let mut vfscache = vfscache;
    for (_i, mod_entry) in orderdata.as_array().unwrap().iter().enumerate() {
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
        .filter(|key| !orderdata.as_array().unwrap().iter().any(|entry| entry["name"].as_str().unwrap_or("") == key.as_str()))
        .cloned()
        .collect();
    for name in stale_keys {
        vfscache.remove(&name);
    }
    for (_i, mod_entry) in orderdata.as_array().unwrap().iter().enumerate() {
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
            let relative_path = pathdiff(&entry_path.to_str().unwrap_or(""), origin_path);
            if entry_path.is_dir() {
                let sub_vfsdata = vfs_scan(entry_path.to_str().unwrap_or(""), origin_path, VFSTree::new());
                vfsdata.insert(relative_path, sub_vfsdata);
            } else {
                let mut file_node = VFSNode::File(VFSFile {
                    paths: OrderedHashMap::new(),
                    dltx_patches: HashMap::new(),
                });
                if let VFSNode::File(ref mut file_data) = file_node {
                    file_data.paths.insert(origin_path.to_string(), entry_path.to_str().unwrap_or("").to_string());
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
                        *existing_tree = merge_vfs_trees(existing_tree.clone(), VFSNode::Dir(new_tree));
                    }
                } else if let VFSNode::File(existing_file) = existing_node {
                    if let VFSNode::File(new_file) = value {
                        existing_file.paths.extend(new_file.paths);
                        existing_file.dltx_patches.extend(new_file.dltx_patches);
                    }
                }
            } else {
                tree1.insert(key, value);
            }
        }
    }
    tree1
}

fn refresh_data(organizesort: OrganizeSort, vfssort: VFSSort, patchdata: Value) -> (Value, VFSTree, HashMap<String, VFSNode>, Vec<OrganizeSortListEntry>, Vec<VFSSortListEntry>) {
    let orderdata = read_order_data();
    let (vfsdata, vfscache) = gen_vfs_data(orderdata.clone(), HashMap::new());
    let organizesort_list = sort_organize_data(orderdata.clone(), organizesort);
    let vfssort_list = gen_vfs_sort_data(vfsdata.clone(), vfssort, patchdata);
    (orderdata, vfsdata, vfscache, organizesort_list, vfssort_list)
}

fn update_data(orderdata: Value, organizesort: OrganizeSort, vfssort: VFSSort, patchdata: Value, vfscache: HashMap<String, VFSNode>) -> (VFSTree, HashMap<String, VFSNode>, Vec<OrganizeSortListEntry>, Vec<VFSSortListEntry>) {
    let (vfsdata, vfscache) = gen_vfs_data(orderdata.clone(), vfscache);
    let text = serde_json::to_string_pretty(&orderdata).expect("Failed to serialize order.json");
    fs::write("configs/order.json", text).expect("Failed to write order.json");
    let organizesort_list = sort_organize_data(orderdata, organizesort);
    let vfssort_list = gen_vfs_sort_data(vfsdata.clone(), vfssort, patchdata);
    (vfsdata, vfscache, organizesort_list, vfssort_list)
}

fn sort_organize_data(orderdata: Value, organizesort: OrganizeSort) -> Vec<OrganizeSortListEntry> {
    let mut orderdata_array = orderdata.as_array().cloned().unwrap_or_else(Vec::new);
    let mut organizesortlist = Vec::new();
    //Add priority field
    for (i, item) in orderdata_array.iter_mut().enumerate() {
        organizesortlist.push(OrganizeSortListEntry {
            enabled: item.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false),
            name: item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            category: item.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            version: item.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            priority: i as i64,
            path: item.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        });
    }
    match organizesort {
        OrganizeSort::NameAsc => organizesortlist.sort_by(|a, b| a.name.cmp(&b.name)),
        OrganizeSort::NameDesc => organizesortlist.sort_by(|a, b| b.name.cmp(&a.name)),
        OrganizeSort::CategoryAsc => organizesortlist.sort_by(|a, b| a.category.cmp(&b.category)),
        OrganizeSort::CategoryDesc => organizesortlist.sort_by(|a, b| b.category.cmp(&a.category)),
        OrganizeSort::PriorityAsc => organizesortlist.sort_by(|a, b| a.priority.cmp(&b.priority)),
        OrganizeSort::PriorityDesc => organizesortlist.sort_by(|a, b| b.priority.cmp(&a.priority)),
    }
    organizesortlist
}

fn install_mod(file_path: &Path) -> Value {
    //TODO Mods: Add mod (complicated)
    //Basic installer (choose root directory) (upgrade with multiple roots)
    //Autodetect root (how)
    //Autodetect multiple roots & prompt (how)
    //fomod Wizard (https://nexus-mods.github.io/NexusMods.App/developers/misc/AboutFomod/)
    //BAIN Wizard (https://wrye-bash.github.io/docs/Wrye%20Bash%20Technical%20Readme.html) (I've never seen this)
    //
    //Strange edgecases: Just look at Anomaly_DevTools. \ is a character somehow??
    //
    //This is going to need to be a window somehow. How?
    Value::Object(serde_json::Map::new())
}

fn read_settings() -> Value {
    let settings_path = Path::new("configs/settings.json");
    if !Path::exists(settings_path) {
        fs::write(settings_path, "{}").expect("Failed to create settings.json");
    }
    let settings_str = fs::read_to_string(settings_path).expect("Failed to read settings.json");
    let settings: Value = serde_json::from_str(&settings_str).expect("Failed to parse settings.json");
    settings
}

fn read_patch_data() -> Value {
    let patchdata_path = Path::new("configs/patch.json");
    if !Path::exists(patchdata_path) {
        fs::write(patchdata_path, "{}").expect("Failed to create patch.json");
    }
    let patchdata_str = fs::read_to_string(patchdata_path).expect("Failed to read patch.json");
    let patchdata: Value = serde_json::from_str(&patchdata_str).expect("Failed to parse patch.json");
    patchdata
}

fn gen_vfs_sort_data(vfsdata: VFSTree, vfssort: VFSSort, patchdata: Value) -> Vec<VFSSortListEntry> {
    let mut vfssortlist = gen_vfs_sort_data_recursive(vfsdata, vfssort.clone(), String::new(), 0, patchdata);
    vfs_sort_data_prune_files(&mut vfssortlist, vfssort.clone());
    vfs_sort_data_prune_folders(&mut vfssortlist, vfssort);
    vfssortlist
}

fn gen_vfs_sort_data_recursive(vfsdata: VFSTree, vfssort: VFSSort, current_path: String, current_down: i64, patchdata: Value) -> Vec<VFSSortListEntry> {
    let mut vfssortlist = Vec::new();
    for (key, value) in vfsdata {
        let name = pathdiff(&key, &current_path);
        if let VFSNode::File(file) = value {
            let file_type = Path::new(&key).extension().and_then(|s| s.to_str()).unwrap_or("file").to_string();
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
                extended: extended,
                file_type: "folder".into(),
                conflicts: 0,
                conflicts_active: 0,
                dltx_patches: 0,
                dltx_patches_active: 0,
            });
            if let VFSNode::Dir(children) = value {
                vfssortlist.extend(gen_vfs_sort_data_recursive(children.clone(), vfssort.clone(), key.clone(), current_down + 1, patchdata.clone()));
            }
        }
    }
    vfssortlist
}

fn vfs_sort_data_prune_files(vfssortlist:&mut Vec<VFSSortListEntry>, vfssort: VFSSort) {
    for i in (0..vfssortlist.len()).rev() {
        let entry = &vfssortlist[i];
        if entry.file_type != "folder" {
            let mut remove = false;
            match vfssort.sort_type {
                VFSSortType::ShowAll => {},
                VFSSortType::ShowConflicts => {
                    if entry.conflicts == 0 {
                        remove = true;
                    }
                },
                VFSSortType::ShowDLTXPatches => {
                    if entry.dltx_patches == 0 {
                        remove = true;
                    }
                },
            }
            if remove {
                vfssortlist.remove(i);
            }
        }
    }
}

fn vfs_sort_data_prune_folders(vfssortlist:&mut Vec<VFSSortListEntry>, vfssort: VFSSort) {
    for i in (0..vfssortlist.len()).rev() {
        let entry = vfssortlist[i].clone();
        if entry.file_type != "folder" {
            continue;
        }

        let has_children = vfssortlist
            .iter()
            .any(|child_entry| is_descendant_path(&child_entry.path, &entry.path));

        let remove_folder = match vfssort.sort_type {
            VFSSortType::ShowAll => false,
            VFSSortType::ShowConflicts | VFSSortType::ShowDLTXPatches => !has_children,
        };

        if remove_folder {
            vfssortlist.remove(i);
        } else if !entry.extended {
            vfssortlist.retain(|child_entry| !is_descendant_path(&child_entry.path, &entry.path));
        }
    }
}

fn is_descendant_path(path: &str, ancestor: &str) -> bool {
    path != ancestor
        && path
            .strip_prefix(ancestor)
            .map(|suffix| suffix.starts_with('/'))
            .unwrap_or(false)
}

fn save_settings(settings: Value) {
    let text = serde_json::to_string_pretty(&settings).expect("Failed to serialize settings.json");
    fs::write("configs/settings.json", text).expect("Failed to write settings.json");
}

fn pathdiff(path: &str, reference: &str) -> String {
    //Does this work on windows?
    if reference.ends_with('/') {
        return path.strip_prefix(reference).unwrap_or(path).to_string();
    } else {
        return path.strip_prefix(&format!("{}/", reference)).unwrap_or(path).to_string();
    }
}

fn realize_vfs_data(vfsdata: VFSTree, patchdata: Value) -> PathBuf {
    //TODO Patch: Realize patchdata

    if fs::metadata(".vfs").is_ok() {
        fs::remove_dir_all(".vfs").expect("Failed to remove existing .vfs directory");
    }
    fs::create_dir(".vfs").expect("Failed to create .vfs directory");
    let vfs_dir = std::env::current_dir().expect("Failed to get current directory").join(".vfs");
    hard_link_vfs_data_recursive(vfsdata, &vfs_dir, patchdata);
    //TODO Launch: Save file structure to .txt
    //TODO Launch: Link saved file structure to .vfs
    vfs_dir
}
fn save_vfs_changes() {
    //TODO Launch: Move any new files to .saved and symlink them back to .vfs
}

fn hard_link_vfs_data_recursive(vfsdata: VFSTree, origin_path: &Path, patchdata: Value) {
    for (key, value) in vfsdata {
        let new_path = origin_path.join(&key);
        if let VFSNode::File(file) = value {
            let source_path = file.paths.iter().last().unwrap().1;
            link_that_file(&PathBuf::from(source_path), &new_path);
        } else if let VFSNode::Dir(children) = value {
            fs::create_dir_all(&new_path).expect("Failed to create directory");
            hard_link_vfs_data_recursive(children, origin_path, patchdata.clone());
        }
    }
}

fn link_that_file(source: &Path, destination: &Path) {
    if let Err(e) = std::os::unix::fs::symlink(source, destination) {
        eprintln!("Failed to create symlink from {:?} to {:?}: {}", source, destination, e);
    }
}