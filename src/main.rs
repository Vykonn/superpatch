use eframe::{egui::{self}};
use serde_json::{Value};
use std::fs;
use serde_json::Map;
use std::path::Path;
use egui_extras::{TableBuilder, Column};

struct SuperPatchApp {
    selected_tab: Tab, 
    orderdata: Value, 
    vfsdata: Value, 
    organizesort: OrganizeSort, 
    organizesort_list: Vec<OrganizeSortListEntry>,
    organizesort_edit: OrganizeSortEdit, 
    status: String,
    settings: Value,
    vfssort: VFSSort,
    vfssort_list: Vec<VFSSortListEntry>,
    vfssort_list_refresh_requested: bool,
    patchdata: Value,
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
    dltx_patches: i64,
    dltx_patches_active: i64,
    patched: bool
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

impl SuperPatchApp {
    fn new(_cc: &eframe::CreationContext<'_>, orderdata: Value, vfsdata: Value, organizesort_list: Vec<OrganizeSortListEntry>, settings: Value, vfssort_list: Vec<VFSSortListEntry>, patchdata: Value) -> Self { 
        Self {selected_tab: Tab::Organize, orderdata, vfsdata, organizesort: OrganizeSort::PriorityAsc, organizesort_list, organizesort_edit: OrganizeSortEdit { edit_type: OrganizeSortEditType::None, index: 0, value: String::new() }, status: "Ready.".to_string(), settings, vfssort: VFSSort { sort_type: VFSSortType::ShowAll, expanded: Vec::new(), query: String::new() }, vfssort_list, vfssort_list_refresh_requested: false, patchdata}
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
                        //TODO: Import from MO2
                        //Modal
                            //MO2 Path
                            //Instance Path
                            //Start/Cancel
                    }
                    if ui.button("Add mod").clicked() {
                        //TODO: Prompt for file path
                        //install_mod(file_path);
                    }
                    if ui.button("Refresh").clicked() {
                        (self.orderdata, self.vfsdata, self.organizesort_list, self.vfssort_list) = refresh_data(self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone());
                        
                    }
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
        //MARK: Right Panel
        egui::Panel::right("right_panel").show(ui, |ui| {
            ui.heading("Superpatch");
            ui.label("v0.1.0");
            if ui.button("Launch").clicked() {
                //TODO: Launch system
            }
        });
        //MARK: Bottom Panel
        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(self.status.clone());
                //TODO: Search bar for organize Tab
                //TODO: Search + Filters for Patch Tab
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
                    //TODO: fix table overflow or underfill
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
                                    (self.vfsdata, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone());
                                    self.status = if enabled { "Enabled mod.".to_string() } else { "Disabled mod.".to_string() };
                                });
                            });
                            row.col(|ui| {
                                if self.organizesort_edit.edit_type == OrganizeSortEditType::Name && self.organizesort_edit.index == row_index {
                                    let response = ui.add(egui::TextEdit::singleline(&mut self.organizesort_edit.value));
                                    if response.lost_focus() {
                                        self.orderdata[priority as usize]["name"] = Value::String(self.organizesort_edit.value.clone());
                                        (self.vfsdata, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone());
                                        self.organizesort_edit.edit_type = OrganizeSortEditType::None;
                                        self.organizesort_edit.index = 0;
                                    }
                                    response.request_focus();
                                } 
                                else {
                                    let response = ui.add(egui::Label::new(name).sense(egui::Sense::click()));
                                    if response.double_clicked() {
                                        self.organizesort_edit.edit_type = OrganizeSortEditType::Name;
                                        self.organizesort_edit.index = row_index;
                                        self.organizesort_edit.value = name.to_string();
                                    }
                                }
                                
                            });
                            row.col(|ui| {
                                if self.organizesort_edit.edit_type == OrganizeSortEditType::Category && self.organizesort_edit.index == row_index {
                                    let response = ui.add(egui::TextEdit::singleline(&mut self.organizesort_edit.value));
                                    if response.lost_focus() {
                                        self.orderdata[priority as usize]["category"] = Value::String(self.organizesort_edit.value.clone());
                                        (self.vfsdata, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone());
                                        self.organizesort_edit.edit_type = OrganizeSortEditType::None;
                                        self.organizesort_edit.index = 0;
                                    }
                                    response.request_focus();
                                } 
                                else {
                                    let response = ui.add(egui::Label::new(category).sense(egui::Sense::click()));
                                    if response.double_clicked() {
                                        self.organizesort_edit.edit_type = OrganizeSortEditType::Category;
                                        self.organizesort_edit.index = row_index;
                                        self.organizesort_edit.value = category.to_string();
                                    }
                                }
                            });
                            row.col(|ui| {
                                if self.organizesort_edit.edit_type == OrganizeSortEditType::Version && self.organizesort_edit.index == row_index {
                                    let response = ui.add(egui::TextEdit::singleline(&mut self.organizesort_edit.value));
                                    if response.lost_focus() {
                                        self.orderdata[priority as usize]["version"] = Value::String(self.organizesort_edit.value.clone());
                                        (self.vfsdata, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone());
                                        self.organizesort_edit.edit_type = OrganizeSortEditType::None;
                                        self.organizesort_edit.index = 0;
                                    }
                                    response.request_focus();
                                } 
                                else {
                                    let response = ui.add(egui::Label::new(version).sense(egui::Sense::click()));
                                    if response.double_clicked() {
                                        self.organizesort_edit.edit_type = OrganizeSortEditType::Version;
                                        self.organizesort_edit.index = row_index;
                                        self.organizesort_edit.value = version.to_string();
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
                                        (self.vfsdata, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone());
                                    }
                                }
                            }
                            response.context_menu(|ui| {
                                if ui.button(if enabled { "Disable" } else { "Enable" }).clicked() {
                                    self.orderdata[row_index]["enabled"] = Value::Bool(!enabled);
                                    (self.vfsdata, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone());
                                    self.status = if enabled { "Disabled mod.".to_string() } else { "Enabled mod.".to_string() };
                                    ui.close();
                                }
                                if ui.button("Copy name").clicked() {
                                    ui.ctx().copy_text(name.to_owned());
                                    self.status = "Copied mod name to clipboard.".to_string();
                                    ui.close();
                                }
                                if ui.button("Rename mod").clicked() {
                                    self.organizesort_edit.edit_type = OrganizeSortEditType::Name;
                                    self.organizesort_edit.index = row_index;
                                    self.organizesort_edit.value = name.to_string();
                                }
                                if ui.button("Edit Category").clicked() {
                                    self.organizesort_edit.edit_type = OrganizeSortEditType::Category;
                                    self.organizesort_edit.index = row_index;
                                    self.organizesort_edit.value = category.to_string();
                                }
                                if ui.button("Change Version").clicked() {
                                    self.organizesort_edit.edit_type = OrganizeSortEditType::Version;
                                    self.organizesort_edit.index = row_index;
                                    self.organizesort_edit.value = version.to_string();
                                }
                                if ui.button("Delete mod").clicked() {
                                    if mod_entry.path.as_str() != "" {
                                    fs::remove_dir_all(mod_entry.path.as_str()).expect("Failed to delete mod directory");
                                    }
                                    self.orderdata.as_array_mut().unwrap().remove(row_index);
                                    (self.vfsdata, self.organizesort_list, self.vfssort_list) = update_data(self.orderdata.clone(), self.organizesort.clone(), self.vfssort.clone(), self.patchdata.clone());
                                    self.status = "Deleted mod.".to_string();
                                    ui.close();
                                }
                                if ui.button("Reinstall mod").clicked() {  
                                    //TODO: Reinstall mod (Store in order.json and re-run installation)
                                    //Also, disable this button if the path is empty or the file doesn't exist.
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
                        //TODO: Scroll when you reach the bottom or top of the screen while dragging
                    }
                }
                //MARK: Files Page
                Tab::Files => {
                    if self.vfssort_list_refresh_requested {
                        self.vfssort_list = gen_vfs_sort_data(self.vfsdata.clone(), self.vfssort.clone(), self.patchdata.clone());
                        self.vfssort_list_refresh_requested = false;
                    }
                    let original_widths = self.settings["files_widths"].as_array().cloned().unwrap_or_else(|| vec![Value::from(200.0), Value::from(100.0), Value::from(100.0)]);
                    //TODO: fix table overflow or underfill
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
                    .body(|mut body |{
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
                                    ui.add_space(down as f32 * 10.0);
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
                                        //Refreshing vfssort_list immediately will crash as the list gets smaller but the UI is still referencing the larger size. Request refresh on next frame.
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
                    //TODO: Patch page
                }
            }
        });
   }
}
//MARK: Main
fn main(){
    if !Path::exists(Path::new(".superpatch")) {
        fs::create_dir(".superpatch").expect("Failed to create superpatch directory");
    }
    let orderdata = read_order_data();
    let vfsdata = gen_vfs_data(orderdata.clone());
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
    let _ = eframe::run_native("Superpatch", native_options, Box::new(|cc| {Ok(Box::new(SuperPatchApp::new(cc, orderdata, vfsdata, organizesort_list, settings, vfssort_list, patchdata)))}));
}

//MARK: Data Handling
fn read_order_data() -> Value {
    let orderdata_path = Path::new(".superpatch/order.json");
    if !Path::exists(orderdata_path) {
        fs::write(orderdata_path, "[]").expect("Failed to create order.json");
    }
    let orderdata_str = fs::read_to_string(orderdata_path).expect("Failed to read order.json");
    let orderdata: Value = serde_json::from_str(&orderdata_str).expect("Failed to parse order.json");
    orderdata
}

fn gen_vfs_data(orderdata: Value) -> Value {
    let mut vfsdata = Value::Object(Map::new());
    for mod_entry in orderdata.as_array().unwrap_or(&vec![]) {
        if mod_entry.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false) {
            let mod_name = mod_entry.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let mod_path_str = mod_entry.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let mod_path = Path::new(mod_path_str);
            vfsdata = read_dir_recursive(mod_path, mod_path, vfsdata.clone(), mod_name);
        }
    }
    vfsdata
}

fn read_dir_recursive(dir_path: &Path, origin_path: &Path, vfsdata: Value, current_mod: &str) -> Value {
    //TODO: Handle DLTX patches
    let mut new_vfsdata = vfsdata;
    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path).expect("Failed to read directory") {
            let entry = entry.expect("Failed to get directory entry");
            let path = entry.path();
            let relative_path = pathdiff::diff_paths(&path, &origin_path).unwrap_or_else(|| path.clone());
            let relative_path_str = relative_path.to_string_lossy().to_string();
            let path_str = path.to_string_lossy().to_string();
            if path.is_dir() {
                if new_vfsdata[relative_path_str.clone()].is_null() {
                    new_vfsdata.as_object_mut().unwrap().insert(relative_path_str.clone(), Value::Object(Map::new()));
                }
                new_vfsdata[relative_path_str] = read_dir_recursive(&path, origin_path, new_vfsdata[relative_path_str.clone()].clone(), current_mod);
            }
            else if path.is_file() {
                if new_vfsdata[relative_path_str.clone()].is_null() {
                    let mut new_file_map = serde_json::Map::new();
                    let mut file_object = serde_json::Map::new();
                    file_object.insert(String::from(current_mod), Value::String(path_str.clone()));
                    new_file_map.insert(String::from("paths"), Value::Object(file_object));
                    new_vfsdata.as_object_mut().unwrap().insert(relative_path_str.clone(), Value::Object(new_file_map));
                } else {
                        new_vfsdata[relative_path_str]["paths"].as_object_mut().unwrap().insert(String::from(current_mod), Value::String(path_str.clone()));
                }
            }
        }
    }
    new_vfsdata
}

fn refresh_data(organizesort: OrganizeSort, vfssort: VFSSort, patchdata: Value) -> (Value, Value, Vec<OrganizeSortListEntry>, Vec<VFSSortListEntry>) {
    let orderdata = read_order_data();
    let vfsdata = gen_vfs_data(orderdata.clone());
    let organizesort_list = sort_organize_data(orderdata.clone(), organizesort);
    let vfssort_list = gen_vfs_sort_data(vfsdata.clone(), vfssort, patchdata);
    (orderdata, vfsdata, organizesort_list, vfssort_list)
}

fn update_data(orderdata: Value, organizesort: OrganizeSort, vfssort: VFSSort, patchdata: Value) -> (Value, Vec<OrganizeSortListEntry>, Vec<VFSSortListEntry>) {
    let vfsdata = gen_vfs_data(orderdata.clone());
    let text = serde_json::to_string_pretty(&orderdata).expect("Failed to serialize order.json");
    fs::write(".superpatch/order.json", text).expect("Failed to write order.json");
    let organizesort_list = sort_organize_data(orderdata, organizesort);
    let vfssort_list = gen_vfs_sort_data(vfsdata.clone(), vfssort, patchdata);
    (vfsdata, organizesort_list, vfssort_list)
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

fn install_mod(file_path: &Path) {
    //TODO: Add mod (complicated)
    //Basic installer (choose root directory) (upgrade with multiple roots)
    //Autodetect root (how)
    //Autodetect multiple roots & prompt (how)
    //fomod Wizard (https://nexus-mods.github.io/NexusMods.App/developers/misc/AboutFomod/)
    //BAIN Wizard (https://wrye-bash.github.io/docs/Wrye%20Bash%20Technical%20Readme.html) (i've never seen this)
    //
    //Strange edgecases: Just look at Anomaly_DevTools. \ is a character somehow??
}

fn read_settings() -> Value {
    let settings_path = Path::new(".superpatch/settings.json");
    if !Path::exists(settings_path) {
        fs::write(settings_path, "{}").expect("Failed to create settings.json");
    }
    let settings_str = fs::read_to_string(settings_path).expect("Failed to read settings.json");
    let settings: Value = serde_json::from_str(&settings_str).expect("Failed to parse settings.json");
    settings
}

fn read_patch_data() -> Value {
    let patchdata_path = Path::new(".superpatch/patch.json");
    if !Path::exists(patchdata_path) {
        fs::write(patchdata_path, "{}").expect("Failed to create patch.json");
    }
    let patchdata_str = fs::read_to_string(patchdata_path).expect("Failed to read patch.json");
    let patchdata: Value = serde_json::from_str(&patchdata_str).expect("Failed to parse patch.json");
    patchdata
}

fn gen_vfs_sort_data(vfsdata: Value, vfssort: VFSSort, patchdata: Value) -> Vec<VFSSortListEntry> {
    //TODO: Generate VFS sort data
    //My name is pseudocode I am here to help
    //Iterate through vfsdata
        //DONE - When you get to a folder, check if it's expanded in vfssort.expanded 
        //DONE - If it is, add it to the list and continue iterating through its children (set file_type to "folder")
        //DONE - If it isn't, add it to the list and skip its children
        //INCM - When you get to a file, add it to the list if it meets the vfssort.sort_type criteria (set file_type to its extension, ("file" if none) set conflicts to the number of conflicts, set dltx_patches to the number of dltx patches)
    //ABSE - If there was a sorting criteria, iterate through the new list.
        //ABSE - If a folder has no children in the list, remove it from the list and go back to the previous item.
    //ABSE - Mark patched files (reference patchdata)
    gen_vfs_sort_data_recursive(vfsdata, vfssort, String::new(), 0, patchdata)
}

fn gen_vfs_sort_data_recursive(vfsdata: Value, vfssort: VFSSort, current_path: String, current_down: i64, patchdata: Value) -> Vec<VFSSortListEntry> {
    let mut vfssortlist = Vec::new();
    for (key, value) in vfsdata.as_object().unwrap_or(&Map::new()) {
        let name = pathdiff::diff_paths(Path::new(key), Path::new(&current_path)).unwrap_or_else(|| Path::new(key).to_path_buf()).to_string_lossy().to_string();
        if value.get("paths").is_some() {
            //It's a file
            let file_type = Path::new(key).extension().and_then(|s| s.to_str()).unwrap_or("file").to_string();
            let mut conflicts = value["paths"].as_object().map(|o| o.len()).unwrap_or(0).try_into().unwrap_or(0);
            if conflicts == 1 {
                conflicts = 0; //If there's only one path, it's not a conflict
            }
            let dltx_patches = value["dltx_patches"].as_array().map(|a| a.len()).unwrap_or(0).try_into().unwrap_or(0);
            //TODO: Check if the file is patched and how many DLTXs remain active (reference patchdata)
            let dltx_patches_active = dltx_patches;
            let patched = false;
            vfssortlist.push(VFSSortListEntry {
                path: key.clone(),
                name,
                down: current_down,
                extended: false,
                file_type,
                conflicts,
                dltx_patches,
                dltx_patches_active,
                patched
            });
        } else {
            //It's a folder
            let extended = vfssort.expanded.contains(&key);
            vfssortlist.push(VFSSortListEntry {
                path: key.clone(),
                name,
                down: current_down,
                extended: extended,
                file_type: "folder".into(),
                conflicts: 0,
                dltx_patches: 0,
                dltx_patches_active: 0,
                patched: false
            });
            if extended {
                vfssortlist.extend(gen_vfs_sort_data_recursive(value.clone(), vfssort.clone(), key.clone(), current_down + 1, patchdata.clone()));
            }
        }
    }
    vfssortlist
}

fn save_settings(settings: Value) {
    let text = serde_json::to_string_pretty(&settings).expect("Failed to serialize settings.json");
    fs::write(".superpatch/settings.json", text).expect("Failed to write settings.json");
}