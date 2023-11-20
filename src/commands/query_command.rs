use super::{QueryCommand, QueryDisplayMode};

impl QueryCommand {
    pub fn new() -> QueryCommand {
        return QueryCommand { 
            pkgs: Vec::new(), 
            xbps_args: Vec::new(),
            display_mode: QueryDisplayMode::Normal 
        };
    }
    pub fn set_display_mode(&mut self, mode: QueryDisplayMode) {
        self.display_mode = mode;
    }
    pub fn execute() {
    }
}
