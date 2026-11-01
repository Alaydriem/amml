use amml_game::ItemStackResponseInfoLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_result_request_id_and_containers_at_the_response_info_offsets() {
    let mut containers = GameMemory::new(0x28);
    containers.put::<u8>(0x00, 28);
    let first = containers.ptr() as usize;
    let mut memory = GameMemory::new(0x30 * 2);
    // ItemStackResponseInfo (48 bytes): mResult 0x00, mClientRequestId 0x08 (vptr, then mRawId at 0x10),
    // mContainers (vector) 0x18
    memory.put::<u8>(0x30, 1);
    memory.put::<usize>(0x30 + 0x08, 0xDEAD_0000);
    memory.put::<i32>(0x30 + 0x10, -15);
    memory.put::<[usize; 3]>(0x30 + 0x18, [first, first + 0x28, first + 0x28]);

    // SAFETY: the buffer is 16-byte aligned and holds two mirrors; the second one's vector points to a live row.
    let responses = unsafe { std::slice::from_raw_parts(memory.ptr().cast::<ItemStackResponseInfoLayout>(), 2) };

    assert_eq!(responses[1].result, 1);
    assert_eq!(responses[1].client_request_id, -15);
    assert_eq!(responses[1].containers.as_slice()[0].container_name, 28);
}
