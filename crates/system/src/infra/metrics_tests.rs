use super::{filter_macos_apfs_volume_group, sampled_at};
use crate::domain::ServerDiskMetrics;

#[test]
fn sampled_at_uses_fixed_utc_milliseconds() {
    let value = sampled_at().unwrap();

    assert_eq!(value.len(), 24);
    assert_eq!(value.as_bytes()[19], b'.');
    assert_eq!(value.as_bytes()[23], b'Z');
}

#[test]
fn apfs_root_and_data_volumes_are_reported_once() {
    let disks = filter_macos_apfs_volume_group(vec![disk("/", "APFS"), disk("/System/Volumes/Data", "apfs"), disk("/Volumes/Archive", "apfs")]);

    let mount_points = disks.iter().map(|disk| disk.mount_point.as_str()).collect::<Vec<_>>();

    assert_eq!(mount_points, vec!["/", "/Volumes/Archive"]);
}

#[test]
fn data_volume_is_preserved_without_an_apfs_root_volume() {
    let disks = filter_macos_apfs_volume_group(vec![disk("/", "ext4"), disk("/System/Volumes/Data", "apfs")]);

    let mount_points = disks.iter().map(|disk| disk.mount_point.as_str()).collect::<Vec<_>>();

    assert_eq!(mount_points, vec!["/", "/System/Volumes/Data"]);
}

fn disk(mount_point: &str, file_system: &str) -> ServerDiskMetrics {
    ServerDiskMetrics {
        name: mount_point.into(),
        mount_point: mount_point.into(),
        file_system: file_system.into(),
        total_bytes: 1_000,
        available_bytes: 100,
        used_bytes: 900,
        usage_percent: 90.0,
    }
}
