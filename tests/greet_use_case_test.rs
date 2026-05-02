// SPDX-License-Identifier: GPL-3.0-or-later
use gtk_cross_platform::core::use_cases::greet_use_case::GreetUseCase;

#[test]
fn returns_greeting() {
    let use_case = GreetUseCase::new();
    assert_eq!(use_case.execute(), "Hello there!");
}
