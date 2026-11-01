use amml_game::ItemStackResponsePacketLayout;

use crate::support::GameMemory;

#[test]
fn walks_responses_containers_and_slots_from_the_packet() {
    let mut slots = GameMemory::new(0x58);
    slots.put::<u8>(0x01, 6);
    slots.put::<i32>(0x04, 4321);
    let slot = slots.ptr() as usize;
    let mut containers = GameMemory::new(0x28);
    containers.put::<u8>(0x00, 29);
    containers.put::<[usize; 3]>(0x10, [slot, slot + 0x58, slot + 0x58]);
    let container = containers.ptr() as usize;
    let mut responses = GameMemory::new(0x30);
    responses.put::<i32>(0x10, 77);
    responses.put::<[usize; 3]>(0x18, [container, container + 0x28, container + 0x28]);
    let response = responses.ptr() as usize;
    let mut memory = GameMemory::new(0x50);
    // PayloadPacket<ItemStackResponsePacketPayload>: mResponses (vector) 0x30
    memory.put::<[usize; 3]>(0x30, [response, response + 0x30, response + 0x30]);

    // SAFETY: the buffer is 16-byte aligned and as large as the mirror; each vector points to live rows.
    let packet = unsafe { &*memory.ptr().cast::<ItemStackResponsePacketLayout>() };

    let response = &packet.responses.as_slice()[0];
    assert_eq!(response.result, 0);
    assert_eq!(response.client_request_id, 77);
    let container = &response.containers.as_slice()[0];
    assert_eq!(container.container_name, 29);
    let slot = &container.slots.as_slice()[0];
    assert_eq!(slot.slot, 6);
    assert_eq!(slot.item_stack_net_id, 4321);
}
