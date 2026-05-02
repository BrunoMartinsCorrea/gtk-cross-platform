// SPDX-License-Identifier: GPL-3.0-or-later
pub struct GreetUseCase;

impl GreetUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self) -> &'static str {
        "Hello there!"
    }
}

impl Default for GreetUseCase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_greeting() {
        let use_case = GreetUseCase::new();
        assert_eq!(use_case.execute(), "Hello there!");
    }
}
