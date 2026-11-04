use amml_launcher_core::{GameStarter, LaunchPlan, LauncherError};
use std::cell::RefCell;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;

// Starts a harmless real process instead of the game and records the plan it was given. Clones share the record.
#[derive(Clone)]
pub struct FakeStarter {
    program: String,
    args: Vec<String>,
    plans: Rc<RefCell<Vec<LaunchPlan>>>,
}

impl FakeStarter {
    // a process that lives for about `seconds` seconds
    pub fn lasting(seconds: u32) -> FakeStarter {
        FakeStarter {
            program: "ping.exe".to_string(),
            args: vec!["-n".to_string(), (seconds + 1).to_string(), "127.0.0.1".to_string()],
            plans: Rc::default(),
        }
    }

    // a process that ends at once with `code`
    pub fn exiting(code: i32) -> FakeStarter {
        FakeStarter {
            program: "cmd.exe".to_string(),
            args: vec!["/c".to_string(), format!("exit {code}")],
            plans: Rc::default(),
        }
    }

    pub fn plans(&self) -> Vec<LaunchPlan> {
        self.plans.borrow().clone()
    }

    // an unrelated process the service did not start
    pub fn stranger() -> Child {
        Command::new("ping.exe")
            .args(["-n", "8", "127.0.0.1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
    }
}

impl GameStarter for FakeStarter {
    fn start(&self, plan: &LaunchPlan) -> Result<Child, LauncherError> {
        self.plans.borrow_mut().push(plan.clone());
        Ok(Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?)
    }
}
