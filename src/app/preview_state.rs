//! Preview scrolling state derived from the current terminal viewport.

#[derive(Debug, Clone, Default)]
pub struct PreviewState {
    pub scroll: u16,
    form_scroll_limit: u16,
    expanded_scroll_limit: u16,
}

impl PreviewState {
    pub fn set_limits(&mut self, form_limit: u16, expanded_limit: Option<u16>, expanded: bool) {
        self.form_scroll_limit = form_limit;
        if let Some(expanded_limit) = expanded_limit {
            self.expanded_scroll_limit = expanded_limit;
        }
        self.scroll = self.scroll.min(self.limit(expanded));
    }

    pub fn can_scroll_up(&self) -> bool {
        self.scroll > 0
    }

    pub fn can_scroll_down(&self, expanded: bool) -> bool {
        self.scroll < self.limit(expanded)
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, expanded: bool) {
        self.scroll = self.scroll.saturating_add(1).min(self.limit(expanded));
    }

    pub fn scroll_to_start(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_end(&mut self, expanded: bool) {
        self.scroll = self.limit(expanded);
    }

    fn limit(&self, expanded: bool) -> u16 {
        if expanded {
            self.expanded_scroll_limit
        } else {
            self.form_scroll_limit
        }
    }
}
