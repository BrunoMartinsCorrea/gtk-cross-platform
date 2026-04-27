// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use gtk_cross_platform::core::use_cases::container_use_case::ContainerUseCase;
use gtk_cross_platform::core::use_cases::image_use_case::ImageUseCase;
use gtk_cross_platform::core::use_cases::network_use_case::NetworkUseCase;
use gtk_cross_platform::core::use_cases::volume_use_case::VolumeUseCase;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;
use gtk_cross_platform::ports::use_cases::i_image_use_case::IImageUseCase;
use gtk_cross_platform::ports::use_cases::i_network_use_case::INetworkUseCase;
use gtk_cross_platform::ports::use_cases::i_volume_use_case::IVolumeUseCase;

fn container_uc() -> impl IContainerUseCase {
    ContainerUseCase::new(Arc::new(MockContainerDriver::new()))
}

fn image_uc() -> impl IImageUseCase {
    ImageUseCase::new(Arc::new(MockContainerDriver::new()))
}

fn volume_uc() -> impl IVolumeUseCase {
    VolumeUseCase::new(Arc::new(MockContainerDriver::new()))
}

fn network_uc() -> impl INetworkUseCase {
    NetworkUseCase::new(Arc::new(MockContainerDriver::new()))
}

#[test]
fn list_with_all_flag_includes_stopped_containers() {
    let uc = container_uc();
    let containers = uc.list(true).expect("list all");
    assert!(!containers.is_empty());
}

#[test]
fn list_containers_running_only() {
    let uc = container_uc();
    let running = uc.list(false).expect("list running");
    assert!(!running.is_empty());
    assert!(running.iter().all(|c| c.status.is_running()));
}

#[test]
fn stop_running_container_removes_from_running_list() {
    let uc = container_uc();
    uc.stop("aabbccdd1122", None).expect("stop");
    let running = uc.list(false).expect("list");
    assert_eq!(
        running.len(),
        0,
        "stopped container must not appear in running list"
    );
}

#[test]
fn start_stopped_container_adds_to_running_list() {
    let uc = container_uc();
    uc.stop("aabbccdd1122", None).expect("stop");
    uc.start("aabbccdd1122").expect("start");
    let running = uc.list(false).expect("list");
    assert_eq!(
        running.len(),
        1,
        "started container must appear in running list"
    );
}

#[test]
fn stop_container_removes_from_running() {
    let uc = container_uc();
    uc.stop("aabbccdd1122", None).expect("stop");
    let running = uc.list(false).expect("list");
    assert_eq!(running.len(), 0);
}

#[test]
fn list_images_returns_images() {
    let uc = image_uc();
    let images = uc.list().expect("list images");
    assert!(images.len() >= 2);
    assert!(images.iter().any(|i| i.primary_tag() == "nginx:latest"));
    assert!(images.iter().any(|i| i.primary_tag() == "postgres:15"));
}

#[test]
fn list_volumes_returns_volumes() {
    let uc = volume_uc();
    let volumes = uc.list().expect("list volumes");
    assert!(!volumes.is_empty());
    assert!(volumes.iter().any(|v| v.name == "postgres-data"));
}

#[test]
fn list_networks_returns_two() {
    let uc = network_uc();
    let networks = uc.list().expect("list networks");
    assert_eq!(networks.len(), 2);
    assert!(networks.iter().any(|n| n.name == "bridge"));
}

#[test]
fn container_stats_returns_values() {
    let uc = container_uc();
    let stats = uc.stats("aabbccdd1122").expect("stats");
    assert!(stats.cpu_percent >= 0.0);
    assert!(stats.memory_usage > 0);
    assert!((stats.memory_usage_mb() - 50.0).abs() < 1.0);
}

#[test]
fn system_df_returns_usage() {
    let uc = network_uc();
    let usage = uc.system_df().expect("df");
    assert_eq!(usage.containers_total, 3, "mock has 3 containers");
    assert_eq!(
        usage.images_total, 3,
        "mock has 3 images (including dangling)"
    );
    assert_eq!(usage.containers_running, 1);
    assert!(usage.containers_total >= usage.containers_running);
}

#[test]
fn prune_system_returns_report() {
    let uc = network_uc();
    let report = uc.prune(false).expect("prune");
    assert_eq!(report.containers_deleted.len(), 1);
}
