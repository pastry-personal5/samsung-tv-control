use samsung_tv_remote::domain::{DeviceDisplay, DeviceId};

#[test]
fn equal_device_ids_compare_equal_and_distinct_ids_do_not() {
    let id1 = DeviceId::new(100);
    let id2 = DeviceId::new(100);
    let id3 = DeviceId::new(200);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn equal_device_ids_address_the_same_map_entry() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let id = DeviceId::new(123);

    map.insert(id, "device");
    assert_eq!(map.get(&DeviceId::new(123)), Some(&"device"));
}

#[test]
fn device_id_formats_as_a_local_record_key() {
    let id = DeviceId::new(999);
    assert_eq!(format!("{}", id), "dev_999");
}

#[test]
fn generated_device_ids_roundtrip_as_local_record_keys() {
    let first = DeviceId::generate();
    let second = DeviceId::generate();
    assert_ne!(first, second);
    assert_eq!(DeviceId::parse_record_key(&first.to_string()), Some(first));
    assert_eq!(
        DeviceId::parse_record_key(&second.to_string()),
        Some(second)
    );
}

#[test]
fn device_display_preserves_its_typed_id() {
    let id = DeviceId::new(100);
    let display = DeviceDisplay::new(id, "Living Room TV");

    assert_eq!(display.id(), id);
    assert_eq!(display.label(), "Living Room TV");
}

#[test]
fn device_display_exposes_its_label_by_borrow() {
    let id = DeviceId::new(200);
    let display = DeviceDisplay::new(id, "Bedroom TV");

    let label: &str = display.label();
    assert_eq!(label, "Bedroom TV");
}

#[test]
fn device_ids_can_be_copied_into_requests() {
    fn takes_copy<T: Copy>(_: T) {}
    takes_copy(DeviceId::new(1));
}

#[test]
fn device_ids_can_be_found_in_a_hash_set() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(DeviceId::new(1));
    assert!(set.contains(&DeviceId::new(1)));
}
