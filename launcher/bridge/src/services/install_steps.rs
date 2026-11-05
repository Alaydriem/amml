use crate::{JobStepStateView, JobStepView, ProgressView};
use amml_gdk::InstallStep;
use amml_launcher_core::CoreProgress;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// The step list of one job and where the job is in it. It turns the core's progress events into steps and one
// byte readout. It words nothing: step ids are fixed words the UI translates.
pub struct InstallSteps {
    steps: Vec<JobStepView>,
    started: Vec<Option<Instant>>,
    // (time, bytes) of the current file; at most `SAMPLE_CAPACITY`, one every `SAMPLE_GAP`
    samples: VecDeque<(Instant, u64)>,
    // done, total and file name of the current download
    bytes: Option<(u64, u64, Option<String>)>,
    // the active mod step has begun installing, so the next download is another mod's
    mod_installing: bool,
    restore_seen: bool,
    // the job is the first run, whose failed fresh copy puts the Store version back
    setup_restore: bool,
    // the step that was in progress when the wait step came in
    paused: Option<usize>,
}

impl InstallSteps {
    pub const SAMPLE_CAPACITY: usize = 16;
    const SAMPLE_GAP: Duration = Duration::from_millis(250);
    const RATE_WINDOW: Duration = Duration::from_secs(5);
    const FINISH: &'static str = "finish";
    const MANAGER_WAIT: &'static str = "manager-wait";
    const SETUP_SETTINGS: &'static str = "setup-settings";
    const COPY_STEPS: [&'static str; 3] = ["setup-check", "setup-measure", "setup-copy"];
    const MOD_PREFIX: &'static str = "mod:";
    const VERSION_STEPS: [&'static str; 9] = [
        "version-check",
        "version-download-current",
        "version-download-wanted",
        "version-install",
        "version-copy-exe",
        "version-copy-files",
        "version-remove",
        "version-restore",
        "version-verify",
    ];

    // the steps of a new instance: the game version (only when it is not in the store yet), the instance folder,
    // AMML, and each mod known beforehand
    pub fn new(version_needed: bool, mod_ids: Vec<String>) -> InstallSteps {
        let mut ids: Vec<String> = Vec::new();
        if version_needed {
            ids.extend(Self::VERSION_STEPS.iter().map(|id| (*id).to_string()));
        }
        ids.extend(["instance", "amml-download", "amml-patch"].map(String::from));
        ids.extend(mod_ids.into_iter().map(|id| format!("{}{id}", Self::MOD_PREFIX)));
        ids.push(Self::FINISH.to_string());
        Self::from_ids(ids)
    }

    pub fn for_mods(mod_ids: Vec<String>) -> InstallSteps {
        let mut ids: Vec<String> = mod_ids.into_iter().map(|id| format!("{}{id}", Self::MOD_PREFIX)).collect();
        ids.push(Self::FINISH.to_string());
        Self::from_ids(ids)
    }

    // updates: each mod, then AMML when it is part of the update
    pub fn for_updates(mod_ids: Vec<String>, with_amml: bool) -> InstallSteps {
        let mut ids: Vec<String> = mod_ids.into_iter().map(|id| format!("{}{id}", Self::MOD_PREFIX)).collect();
        if with_amml {
            ids.extend(["amml-download", "amml-patch"].map(String::from));
        }
        ids.push(Self::FINISH.to_string());
        Self::from_ids(ids)
    }

    pub fn for_amml() -> InstallSteps {
        Self::from_ids(["amml-download", "amml-patch", Self::FINISH].map(String::from).to_vec())
    }

    // The first run: check the target, measure, copy, then save the settings. A start with no worlds has only the
    // last step.
    pub fn for_setup(copy: bool) -> InstallSteps {
        let mut ids: Vec<String> = Vec::new();
        if copy {
            ids.extend(Self::COPY_STEPS.map(String::from));
        }
        ids.push(Self::SETUP_SETTINGS.to_string());
        let mut steps = Self::from_ids(ids);
        // nothing is copied before the measure step, so a failure before it changes nothing
        steps.setup_restore = copy;
        steps.restore_seen = copy;
        steps
    }

    // a copy of worlds on its own: the setup steps without the settings
    pub fn for_copy() -> InstallSteps {
        Self::from_ids(Self::COPY_STEPS.map(String::from).to_vec())
    }

    // one named step and the finish
    pub fn single(id: &str) -> InstallSteps {
        Self::from_ids(vec![id.to_string(), Self::FINISH.to_string()])
    }

    fn from_ids(ids: Vec<String>) -> InstallSteps {
        let steps: Vec<JobStepView> = ids
            .into_iter()
            .map(|id| JobStepView { id, state: JobStepStateView::Waiting, millis: None, cached: false, percent: None })
            .collect();
        InstallSteps {
            started: vec![None; steps.len()],
            steps,
            samples: VecDeque::with_capacity(Self::SAMPLE_CAPACITY),
            bytes: None,
            mod_installing: false,
            restore_seen: false,
            setup_restore: false,
            paused: None,
        }
    }

    // the first step is the one in progress when the job starts
    pub fn start(&mut self, now: Instant) {
        if !self.steps.is_empty() {
            self.activate(0, now);
        }
    }

    // `true` when a step began, ended or was added
    pub fn apply(&mut self, event: &CoreProgress, now: Instant) -> bool {
        match event {
            CoreProgress::InstallingVersion { step, download } => {
                if *step == InstallStep::Restoring {
                    self.restore_seen = true;
                }
                let changed = if *step == InstallStep::Done {
                    self.complete_active(now)
                } else {
                    match self.index_of(Self::version_id(*step)) {
                        Some(index) => self.activate(index, now),
                        None => false,
                    }
                };
                if let Some(download) = download {
                    self.record(download.bytes_done, download.bytes_total, None, now);
                }
                changed
            }
            CoreProgress::CreatingInstance => self.activate_named("instance", now),
            CoreProgress::Downloading { file, progress } => {
                let changed = match self.download_target() {
                    Some(index) => {
                        self.mod_installing = false;
                        self.activate(index, now)
                    }
                    None => false,
                };
                self.record(progress.bytes_done, progress.bytes_total, Some(file.clone()), now);
                changed
            }
            CoreProgress::Verifying { .. } => false,
            CoreProgress::InstallingAmml { .. } | CoreProgress::Patching => self.activate_named("amml-patch", now),
            CoreProgress::InstallingMod { id } => {
                let name = format!("{}{id}", Self::MOD_PREFIX);
                let added = self.index_of(&name).is_none();
                if added {
                    self.insert_before_finish(name.clone());
                }
                self.mod_installing = true;
                let activated = self.activate_named(&name, now);
                added || activated
            }
            CoreProgress::Finishing => self.activate_named(Self::FINISH, now),
            CoreProgress::Done => {
                self.finish(now);
                true
            }
            CoreProgress::WaitingForManager { waiting } => self.set_waiting(*waiting, now),
            CoreProgress::MeasuringData { merge } => {
                self.restore_seen = self.setup_restore && !*merge;
                self.activate_named("setup-measure", now)
            }
            CoreProgress::CopyingData { detail, progress } => {
                let changed = self.activate_named("setup-copy", now);
                self.record(progress.bytes_done, progress.bytes_total, Some(detail.clone()), now);
                changed
            }
            CoreProgress::SavingSettings => self.activate_named(Self::SETUP_SETTINGS, now),
        }
    }

    // A job that waits for the manager shows a first step for it; the step goes when the job has the manager. The
    // step is not part of the job: it is not counted, and only a step that was in progress goes on afterwards.
    fn set_waiting(&mut self, waiting: bool, now: Instant) -> bool {
        let present = self.steps.first().is_some_and(|step| step.id == Self::MANAGER_WAIT);
        if waiting && !present {
            self.paused = self.active_index();
            if let Some(index) = self.paused {
                self.steps[index].state = JobStepStateView::Waiting;
            }
            let step = JobStepView {
                id: Self::MANAGER_WAIT.to_string(),
                state: JobStepStateView::Active,
                millis: None,
                cached: false,
                percent: None,
            };
            self.steps.insert(0, step);
            self.started.insert(0, Some(now));
            return true;
        }
        if !waiting && present {
            self.steps.remove(0);
            self.started.remove(0);
            if let Some(index) = self.paused.take() {
                self.steps[index].state = JobStepStateView::Active;
                self.started[index] = Some(now);
            }
            return true;
        }
        false
    }

    // 1 when the first step is the wait step, which is not counted
    fn waiting_offset(&self) -> usize {
        usize::from(self.steps.first().is_some_and(|step| step.id == Self::MANAGER_WAIT))
    }

    // the job ended well: every step that has not failed is done
    pub fn finish(&mut self, now: Instant) {
        for index in 0..self.steps.len() {
            match self.steps[index].state {
                JobStepStateView::Active => self.complete(index, now),
                JobStepStateView::Waiting => self.steps[index].state = JobStepStateView::Done,
                _ => {}
            }
        }
        self.bytes = None;
        self.samples.clear();
    }

    // the step in progress is the one that failed
    pub fn fail(&mut self) {
        if let Some(index) = self.active_index() {
            self.steps[index].state = JobStepStateView::Failed;
            self.steps[index].millis = None;
        }
    }

    pub fn steps(&self) -> Vec<JobStepView> {
        self.steps.clone()
    }

    pub fn progress(&self) -> Option<ProgressView> {
        let (done, total, file) = self.bytes.as_ref()?;
        let rate = self.rate();
        let remaining_seconds = rate.filter(|rate| *rate > 0).map(|rate| total.saturating_sub(*done) / rate);
        Some(ProgressView { done: *done, total: *total, rate, remaining_seconds, file: file.clone(), mirror: None })
    }

    pub fn restore_seen(&self) -> bool {
        self.restore_seen
    }

    pub fn total(&self) -> u32 {
        (self.steps.len() - self.waiting_offset()) as u32
    }

    // 1-based number of the step in progress; the count of finished steps when none is
    pub fn active_number(&self) -> u32 {
        match self.active_index() {
            // the wait step stands for the first step that has not begun
            Some(index) if self.steps[index].id == Self::MANAGER_WAIT => 1,
            Some(index) => (index - self.waiting_offset()) as u32 + 1,
            None => self.steps.iter().filter(|step| step.state == JobStepStateView::Done).count() as u32,
        }
    }

    // how many byte samples are held
    pub fn samples_held(&self) -> usize {
        self.samples.len()
    }

    fn version_id(step: InstallStep) -> &'static str {
        match step {
            InstallStep::Checking => "version-check",
            InstallStep::DownloadingCurrent => "version-download-current",
            InstallStep::DownloadingWanted => "version-download-wanted",
            InstallStep::Installing => "version-install",
            InstallStep::CopyingExe => "version-copy-exe",
            InstallStep::CopyingFiles => "version-copy-files",
            InstallStep::Removing => "version-remove",
            InstallStep::Restoring => "version-restore",
            InstallStep::Verifying => "version-verify",
            InstallStep::Done => "version-verify",
        }
    }

    fn index_of(&self, id: &str) -> Option<usize> {
        self.steps.iter().position(|step| step.id == id)
    }

    fn active_index(&self) -> Option<usize> {
        self.steps.iter().position(|step| step.state == JobStepStateView::Active)
    }

    fn is_download(id: &str) -> bool {
        id.starts_with("version-download") || id == "amml-download"
    }

    // The step a download belongs to. A mod zip belongs to the mod step that is active and has not begun installing.
    // Any other zip belongs to the first step in line that takes one: `amml-download` or a mod step that waits.
    fn download_target(&self) -> Option<usize> {
        if let Some(index) = self.active_index()
            && self.steps[index].id.starts_with(Self::MOD_PREFIX)
            && !self.mod_installing
        {
            return Some(index);
        }
        self.steps.iter().position(|step| {
            step.state == JobStepStateView::Waiting
                && (step.id == "amml-download" || step.id.starts_with(Self::MOD_PREFIX))
        })
    }

    fn insert_before_finish(&mut self, id: String) {
        let at = self.index_of(Self::FINISH).unwrap_or(self.steps.len());
        let step = JobStepView { id, state: JobStepStateView::Waiting, millis: None, cached: false, percent: None };
        self.steps.insert(at, step);
        self.started.insert(at, None);
    }

    fn activate_named(&mut self, id: &str, now: Instant) -> bool {
        match self.index_of(id) {
            Some(index) => self.activate(index, now),
            None => false,
        }
    }

    // Makes the step the one in progress. Whatever was in progress ends; download steps before it that never ran
    // were served from disk.
    fn activate(&mut self, index: usize, now: Instant) -> bool {
        if self.steps[index].state == JobStepStateView::Active {
            return false;
        }
        for other in 0..self.steps.len() {
            if other != index && self.steps[other].state == JobStepStateView::Active {
                self.complete(other, now);
            }
        }
        for earlier in 0..index {
            if self.steps[earlier].state == JobStepStateView::Waiting && Self::is_download(&self.steps[earlier].id) {
                self.steps[earlier].state = JobStepStateView::Done;
                self.steps[earlier].cached = true;
            }
        }
        self.steps[index].state = JobStepStateView::Active;
        self.steps[index].percent = None;
        self.started[index] = Some(now);
        self.bytes = None;
        self.samples.clear();
        true
    }

    fn complete_active(&mut self, now: Instant) -> bool {
        match self.active_index() {
            Some(index) => {
                self.complete(index, now);
                true
            }
            None => false,
        }
    }

    fn complete(&mut self, index: usize, now: Instant) {
        let step = &mut self.steps[index];
        step.state = JobStepStateView::Done;
        step.percent = None;
        step.millis = self.started[index].map(|began| now.saturating_duration_since(began).as_millis() as u64);
    }

    fn record(&mut self, done: u64, total: u64, file: Option<String>, now: Instant) {
        let file = file.or_else(|| self.bytes.as_ref().and_then(|(_, _, held)| held.clone()));
        self.bytes = Some((done, total, file));
        if let Some(index) = self.active_index() {
            self.steps[index].percent = (total > 0).then(|| (done.saturating_mul(100) / total).min(100) as u32);
        }
        if let Some((_, last_bytes)) = self.samples.back()
            && done < *last_bytes
        {
            self.samples.clear();
        }
        let due = self.samples.back().is_none_or(|(time, _)| now.saturating_duration_since(*time) >= Self::SAMPLE_GAP);
        if due {
            if self.samples.len() == Self::SAMPLE_CAPACITY {
                self.samples.pop_front();
            }
            self.samples.push_back((now, done));
        }
    }

    // bytes a second between the newest sample and the oldest one within the window
    fn rate(&self) -> Option<u64> {
        let (latest_time, latest_bytes) = *self.samples.back()?;
        let (oldest_time, oldest_bytes) =
            *self.samples.iter().find(|(time, _)| latest_time.saturating_duration_since(*time) <= Self::RATE_WINDOW)?;
        let elapsed = latest_time.saturating_duration_since(oldest_time).as_secs_f64();
        if elapsed <= 0.0 {
            return None;
        }
        Some((latest_bytes.saturating_sub(oldest_bytes) as f64 / elapsed) as u64)
    }
}
