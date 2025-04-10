use crate::{
    class::ClassList, config::YClassConfig, context::Selection, hotkeys::HotkeyManager,
    process::YProcess, project::ProjectData,
};
use egui_notify::Toasts;
use parking_lot::RwLock;
use std::{
    cell::RefCell,
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use memflow::prelude::v1::*;

pub type StateRef = &'static RefCell<GlobalState>;

pub struct GlobalState {
    pub last_opened_project: Option<PathBuf>,
    pub selection: Option<Selection>,
    pub inventory: Inventory,
    pub os: Arc<RwLock<Option<OsInstanceArcBox<'static>>>>,
    pub process: Arc<RwLock<Option<YProcess>>>,
    pub hotkeys: HotkeyManager,
    pub class_list: ClassList,
    pub config: YClassConfig,
    pub toasts: Toasts,
    /// `true` means project was just created and contains no useful
    /// information
    pub dummy: bool,
}

impl Default for GlobalState {
    fn default() -> Self {
        let config = YClassConfig::load_or_default();

        let inventory = Inventory::scan();

        Self {
            hotkeys: HotkeyManager::default(),
            class_list: ClassList::default(),
            last_opened_project: None,
            toasts: Toasts::default(),
            inventory,
            os: Arc::default(),
            process: Arc::default(),
            selection: None,
            dummy: true,
            config,
        }
    }
}

impl GlobalState {
    pub fn save_project_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save current project")
            .add_filter("YClass project", &["yclass"])
            .save_file()
        {
            self.save_project(Some(&path));
        }
    }

    pub fn save_project(&mut self, path: Option<&Path>) {
        if let Some(path) = path {
            let pd = ProjectData::store(self.class_list.classes()).to_string();
            if let Err(e) = fs::write(path, pd.as_bytes()) {
                self.toasts
                    .error(format!("Failed to save the project. {e}"));
            } else {
                self.last_opened_project = Some(path.to_owned());
                self.dummy = false;
            }
        } else if let Some(ref last) = self.last_opened_project {
            let pd = ProjectData::store(self.class_list.classes()).to_string();
            if let Err(e) = fs::write(last, pd.as_bytes()) {
                self.toasts
                    .error(format!("Failed to save the project. {e}"));
            } else {
                self.last_opened_project = Some(last.to_owned());
                self.dummy = false;
            }
        } else if let Some(path) = rfd::FileDialog::new()
            .set_title("Save current project")
            .add_filter("YClass project", &["yclass"])
            .save_file()
        {
            self.save_project(Some(&path));
        }
    }

    pub fn open_project(&mut self) -> bool {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open existing project")
            .add_filter("YClass project", &["yclass"])
            .pick_file()
        {
            self.open_project_path(&path)
        } else {
            true
        }
    }

    pub fn open_project_path(&mut self, path: &Path) -> bool {
        if !self.class_list.classes().is_empty() && !self.dummy {
            self.save_project(None);
        }

        match fs::read_to_string(path) {
            Ok(data) => {
                if let Some(pd) = ProjectData::from_str(&data) {
                    self.class_list = pd.load();
                    self.dummy = false;
                    self.last_opened_project = Some(path.to_path_buf());

                    if let Some(recent) = self.config.recent_projects.as_mut() {
                        recent.insert(path.to_path_buf());
                    } else {
                        self.config.recent_projects =
                            Some(HashSet::from_iter([path.to_path_buf()]));
                    }
                    self.config.save();

                    true
                } else {
                    self.toasts.error("Project file is in invalid format");
                    false
                }
            }
            Err(e) => {
                self.toasts
                    .error(format!("Failed to open the project. {e}"));
                false
            }
        }
    }
}
