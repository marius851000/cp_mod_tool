use console::style;

pub struct StepProgress {
    step_number: u32,
    progress: u32,
}

impl StepProgress {
    pub fn new(step_number: u32) -> Self {
        Self {
            step_number,
            progress: 0,
        }
    }

    pub fn progress(&mut self, message: &str) {
        self.progress += 1;
        self.print_progress(message)
    }

    fn print_progress(&self, message: &str) {
        println!(
            "{} {}",
            style(format!("[{}/{}]", self.progress, self.step_number))
                .bold()
                .dim(),
            message
        );
    }
}
