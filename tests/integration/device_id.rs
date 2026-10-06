use samsung_tv_remote::domain::{DeviceDisplay, DeviceId};

#[test]
fn device_id_equality() {
    let id1 = DeviceId::new(100);
    let id2 = DeviceId::new(100);
    let id3 = DeviceId::new(200);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn device_id_hash_consistency() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let id = DeviceId::new(123);

    map.insert(id, "device");
    assert_eq!(map.get(&DeviceId::new(123)), Some(&"device"));
}

#[test]
fn device_id_display() {
    let id = DeviceId::new(999);
    assert_eq!(format!("{}", id), "dev_999");
}

#[test]
fn device_display_preserves_its_typed_id() {
    let id = DeviceId::new(100);
    let display = DeviceDisplay::new(id, "Living Room TV");

    assert_eq!(display.id(), id);
    assert_eq!(display.label(), "Living Room TV");
}

#[test]
fn device_display_label_borrow() {
    let id = DeviceId::new(200);
    let display = DeviceDisplay::new(id, "Bedroom TV");

    let label: &str = display.label();
    assert_eq!(label, "Bedroom TV");
}

#[test]
fn device_id_is_copy() {
    fn takes_copy<T: Copy>(_: T) {}
    takes_copy(DeviceId::new(1));
}

#[test]
fn device_id_is_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(DeviceId::new(1));
    assert!(set.contains(&DeviceId::new(1)));
}
