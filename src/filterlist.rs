use crate::tui::StateL;

pub struct FilterList {
    pub state: StateL,
    pub list: Vec<String>,
    pub filter: String,
}

impl FilterList {
    pub fn new(list: Vec<String>, filter: String) -> Self {
        Self {
            state: StateL::new(list.len()),
            list,
            filter,
        }
    }

    pub fn get_filtered(&mut self) -> Vec<String> {
        let l: Vec<String> = self
            .list
            .clone()
            .into_iter()
            .filter(|rn| rn.to_lowercase().contains(&self.filter.to_lowercase()))
            .collect();
        self.state.new_size(l.len());
        return l;
    }

    pub fn set_list(&mut self, new_list: Vec<String>) {
        self.list = new_list;
        self.set_filter("".to_string());
        self.state.state = ratatui::widgets::ListState::default();
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
    }

    pub fn filter_append(&mut self, c: char) {
        self.filter.push(c);
    }

    pub fn filter_remove_last(&mut self) {
        self.filter.pop();
    }

    pub fn get_index(&self) -> Option<usize> {
        return self.state.get_selected_index();
    }
}
