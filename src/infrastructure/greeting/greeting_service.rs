// SPDX-License-Identifier: GPL-3.0-or-later
use crate::core::use_cases::greet_use_case::GreetUseCase;
use crate::ports::i_greeting_service::IGreetingService;

pub struct GreetingService {
    use_case: GreetUseCase,
}

impl GreetingService {
    pub fn new() -> Self {
        Self {
            use_case: GreetUseCase::new(),
        }
    }
}

impl Default for GreetingService {
    fn default() -> Self {
        Self::new()
    }
}

impl IGreetingService for GreetingService {
    fn greet(&self) -> String {
        self.use_case.execute().to_string()
    }
}
