use amml_derive::testing::ImageFixture;
use amml_derive::{Image, PacketRow, PacketTable};

const BASE: u64 = 0x1_4000_0000;

struct Packet {
    id: u32,
    name: &'static str,
    // the make_shared allocation: the packet size plus the 10h control block; `None` leaves the `mov edx` out
    alloc: Option<u32>,
    slot: u32,
    // the case function calls a constructor that loads the vtable, as most frame-pointer case functions do
    in_ctor: bool,
}

/// A `createPacket` in the MSVC form: `cmp edx, n-1; ja default; mov eax, edx; lea rcx, [table];
/// movsxd rax, [rcx+rax*4]; add rax, rcx; jmp rax`, then a default `ret` and one `mov rcx, rsi; call case; ret`
/// stub per packet. The last case of the table goes to the default.
struct PacketFixture {
    fx: ImageFixture,
    rdata_end: u32,
    text_end: u32,
}

impl PacketFixture {
    fn new() -> Self {
        Self { fx: ImageFixture::new(BASE), rdata_end: 0x40_0000, text_end: 0x1000 }
    }

    fn rdata(&mut self, bytes: &[u8]) -> u32 {
        let at = self.fx.put_rdata(bytes);
        self.rdata_end = at + bytes.len() as u32;
        at
    }

    fn text(&mut self, bytes: &[u8]) -> u32 {
        let at = self.fx.put_text(bytes);
        self.text_end = at + bytes.len() as u32;
        at
    }

    fn next_text(&self) -> u32 {
        self.text_end.next_multiple_of(16)
    }

    fn rel(from_next: u32, target: u32) -> [u8; 4] {
        target.wrapping_sub(from_next).to_le_bytes()
    }

    fn qword(rva: u32) -> [u8; 8] {
        (BASE + u64::from(rva)).to_le_bytes()
    }

    // returns the case function of one packet
    fn put_packet(&mut self, packet: &Packet) -> u32 {
        let name = self.rdata(packet.name.as_bytes());
        let get_id = self.text(&[[0xB8].as_slice(), &packet.id.to_le_bytes(), &[0xC3]].concat());
        // mov rax, rdx; lea rcx, [rip+name]; mov [rdx], rcx; mov qword ptr [rdx+8], len; ret
        let g = self.next_text();
        let mut get_name = vec![0x48, 0x89, 0xD0, 0x48, 0x8D, 0x0D];
        get_name.extend(Self::rel(g + 10, name));
        get_name.extend([0x48, 0x89, 0x0A, 0x48, 0xC7, 0x42, 0x08]);
        get_name.extend((packet.name.len() as u32).to_le_bytes());
        get_name.push(0xC3);
        assert_eq!(self.text(&get_name), g);
        let dtor = self.text(&[0xC3]);
        // mov rcx, r8; mov r8, [r9]; mov rax, [rcx]; mov rax, [rax+slot*8]; jmp rax
        let mut thunk = vec![0x4C, 0x89, 0xC1, 0x4D, 0x8B, 0x01, 0x48, 0x8B, 0x01, 0x48, 0x8B, 0x80];
        thunk.extend((packet.slot * 8).to_le_bytes());
        thunk.extend([0xFF, 0xE0]);
        let thunk = self.text(&thunk);
        let vtable = self.rdata(&[Self::qword(dtor), Self::qword(get_id), Self::qword(g)].concat());
        let handler_vtable = self.rdata(&[Self::qword(dtor), Self::qword(thunk)].concat());
        let handler = self.fx.put_data(&Self::qword(handler_vtable));
        // with `in_ctor`, a constructor `lea rcx, [rip+vtable]; ret` that the case function calls
        let ctor = if packet.in_ctor {
            let at = self.next_text();
            let mut ctor = vec![0x48, 0x8D, 0x0D];
            ctor.extend(Self::rel(at + 7, vtable));
            ctor.push(0xC3);
            assert_eq!(self.text(&ctor), at);
            Some(at)
        } else {
            None
        };
        // [mov edx, alloc;] call get_id or the constructor; [lea rcx, [rip+vtable];] lea rdx, [rip+handler]; ret
        let c = self.next_text();
        let mut case = Vec::new();
        if let Some(alloc) = packet.alloc {
            case.push(0xBA);
            case.extend(alloc.to_le_bytes());
        }
        let call_at = c + case.len() as u32;
        case.push(0xE8);
        case.extend(Self::rel(call_at + 5, ctor.unwrap_or(get_id)));
        if ctor.is_none() {
            let lea_at = c + case.len() as u32;
            case.extend([0x48, 0x8D, 0x0D]);
            case.extend(Self::rel(lea_at + 7, vtable));
        }
        let lea_at = c + case.len() as u32;
        case.extend([0x48, 0x8D, 0x15]);
        case.extend(Self::rel(lea_at + 7, handler));
        case.push(0xC3);
        assert_eq!(self.text(&case), c);
        self.fx.put_function(c, c + case.len() as u32);
        c
    }

    // returns the createPacket RVA
    fn put_create_packet(&mut self, cases: &[u32]) -> u32 {
        let count = cases.len() as u32 + 1;
        let table = self.rdata_end.next_multiple_of(16);
        let p = self.next_text();
        let mut code = vec![0x81, 0xFA];
        code.extend((count - 1).to_le_bytes());
        code.extend([0x77, 0x12, 0x89, 0xD0, 0x48, 0x8D, 0x0D]);
        code.extend(Self::rel(p + 17, table));
        code.extend([0x48, 0x63, 0x04, 0x81, 0x48, 0x01, 0xC8, 0xFF, 0xE0]);
        let default = p + code.len() as u32;
        code.push(0xC3);
        let mut entries = Vec::new();
        for case in cases {
            let stub = p + code.len() as u32;
            code.extend([0x48, 0x89, 0xF1, 0xE8]);
            code.extend(Self::rel(stub + 8, *case));
            code.push(0xC3);
            entries.push(stub);
        }
        entries.push(default);
        assert_eq!(self.text(&code), p);
        self.fx.put_function(p, p + code.len() as u32);
        let bytes: Vec<u8> = entries.iter().flat_map(|stub| (stub.wrapping_sub(table) as i32).to_le_bytes()).collect();
        assert_eq!(self.rdata(&bytes), table);
        p
    }

    fn image(&self) -> Image {
        self.fx.image()
    }
}

fn table_of(packets: &[Packet]) -> (Vec<PacketRow>, String) {
    let mut fx = PacketFixture::new();
    let cases: Vec<u32> = packets.iter().map(|packet| fx.put_packet(packet)).collect();
    let create = fx.put_create_packet(&cases);
    let image = fx.image();
    let table = match PacketTable::read(&image, create) {
        Ok(table) => table,
        Err(error) => panic!("{error}"),
    };
    let text = PacketTable::render(&table, None);
    (table.rows, text)
}

#[test]
fn packet_table_reads_jump_table_cases() {
    let (rows, text) = table_of(&[
        Packet { id: 328, name: "PrimitiveShapesPacket", alloc: Some(0x158), slot: 33, in_ctor: false },
        Packet { id: 300, name: "CameraInstructionPacket", alloc: Some(0x198), slot: 147, in_ctor: false },
    ]);
    let summary: Vec<_> = rows.iter().map(|row| (row.id, row.name.clone(), row.size, row.slot)).collect();
    assert_eq!(
        summary,
        vec![
            (300, Some("CameraInstructionPacket".to_string()), Some(0x188), Some(147)),
            (328, Some("PrimitiveShapesPacket".to_string()), Some(0x148), Some(33)),
        ]
    );
    assert!(text.ends_with("cases: 3, rows: 2, empty: 1, skipped: 0"), "{text}");
}

#[test]
fn packet_row_without_size_prints_question_mark() {
    let (rows, text) = table_of(&[Packet { id: 7, name: "TextPacket", alloc: None, slot: 12, in_ctor: false }]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].size, None);
    let line = text.lines().find(|line| line.contains("TextPacket")).unwrap_or_default();
    assert!(line.contains("  ?  "), "{line}");
}

#[test]
fn packet_vtable_found_one_call_deeper() {
    let (rows, text) =
        table_of(&[Packet { id: 77, name: "SetTimePacket", alloc: Some(0x48), slot: 20, in_ctor: true }]);
    let summary: Vec<_> = rows.iter().map(|row| (row.id, row.name.clone(), row.size, row.slot)).collect();
    assert_eq!(summary, vec![(77, Some("SetTimePacket".to_string()), Some(0x38), Some(20))]);
    assert!(text.ends_with("skipped: 0"), "{text}");
}
