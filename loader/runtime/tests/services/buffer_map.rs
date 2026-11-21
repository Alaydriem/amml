use amml_abi::{AmmlPoint, AmmlSize};
use amml_runtime::BufferMap;

fn size(width: u32, height: u32) -> AmmlSize {
    AmmlSize { width, height }
}

#[test]
fn a_point_maps_to_the_forced_buffer_size() {
    let map = BufferMap::new();
    map.set_client(size(1000, 500));
    map.set_buffer(size(2000, 1000));
    assert_eq!(map.to_buffer(AmmlPoint { x: 10.0, y: 10.0 }), Some(AmmlPoint { x: 20.0, y: 20.0 }));
    map.set_client(size(0, 0));
    assert_eq!(map.to_buffer(AmmlPoint { x: 10.0, y: 10.0 }), None);
}

#[test]
fn no_buffer_size_gives_none() {
    let map = BufferMap::new();
    map.set_client(size(1000, 500));
    assert_eq!(map.to_buffer(AmmlPoint { x: 1.0, y: 1.0 }), None);
    map.set_buffer(size(1000, 0));
    assert_eq!(map.to_buffer(AmmlPoint { x: 1.0, y: 1.0 }), None);
}

#[test]
fn a_window_that_is_resized_maps_by_its_new_size() {
    let map = BufferMap::new();
    map.set_client(size(1000, 500));
    map.set_buffer(size(1000, 500));
    map.set_client(size(500, 250));
    assert_eq!(map.to_buffer(AmmlPoint { x: 10.0, y: 5.0 }), Some(AmmlPoint { x: 20.0, y: 10.0 }));
}
