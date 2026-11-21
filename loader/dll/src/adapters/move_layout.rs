use amml_abi::AmmlMoveIntent;

/// Where the move fields sit in the game's `RawMoveInputComponent`, from the ten derived
/// `RawMoveInputComponent.*` and `MoveInputState.*` values. Offsets are in bytes: `input_state` and
/// `raw_move` from the start of the component, `analog` from the start of the `MoveInputState`. The
/// flag word is the first `u32` of the `MoveInputState`. `sneak_mask` is the game's sneak test, which
/// also holds the Descend bit; only SneakDown (bit 0, `SNEAK_DOWN`) is read and written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveLayout {
    pub input_state: u32,
    pub raw_move: u32,
    pub analog: u32,
    pub sneak_mask: u32,
    pub jump_mask: u32,
    pub sprint_bit: u32,
    pub up_bit: u32,
    pub down_bit: u32,
    pub left_bit: u32,
    pub right_bit: u32,
}

impl MoveLayout {
    /// The size of `RawMoveInputComponent` on every build the entries derive on; no read or write
    /// leaves it.
    pub const COMPONENT_SIZE: usize = 0x18;
    /// The ten entries, in the order of the fields.
    pub const ENTRIES: [&'static str; 10] = [
        "RawMoveInputComponent.mInputState",
        "RawMoveInputComponent.mRawMove",
        "MoveInputState.mAnalogMoveVector",
        "MoveInputState.maskSneakTest",
        "MoveInputState.maskJumpDown",
        "MoveInputState.bitSprintDown",
        "MoveInputState.bitUp",
        "MoveInputState.bitDown",
        "MoveInputState.bitLeft",
        "MoveInputState.bitRight",
    ];
    /// An axis past this, either way, also sets its direction bit; below it only the vector moves.
    pub const DIRECTION_DEAD_ZONE: f32 = 0.3;
    /// SneakDown, the one bit of the sneak test that is read and written.
    pub const SNEAK_DOWN: u32 = 1;
    /// The jump mask the game uses on every build the entries derive on.
    pub const EXPECTED_JUMP_MASK: u32 = 0x80;
    /// The SprintDown, Up, Down, Left and Right bit numbers the game uses on every build the entries
    /// derive on.
    pub const EXPECTED_BITS: [u32; 5] = [8, 13, 14, 15, 16];
    const FLAGS_SIZE: u32 = 4;
    const VEC2_SIZE: u32 = 8;
    const OWNED_BITS: u32 = 7;

    /// The layout from the ten entries `resolve` gives, when each resolves, fits in a `u32` and the
    /// layout passes the bounds check: every read and write stays inside the component, the flag
    /// word and the two vectors do not overlap, every bit number is below 32, the jump mask is one
    /// bit and the seven owned bits are distinct.
    pub fn from_entries(resolve: impl Fn(&str) -> Option<u64>) -> Option<MoveLayout> {
        let mut values = [0u32; 10];
        for (value, entry) in values.iter_mut().zip(Self::ENTRIES) {
            *value = u32::try_from(resolve(entry)?).ok()?;
        }
        let [input_state, raw_move, analog, sneak_mask, jump_mask, sprint_bit, up_bit, down_bit, left_bit, right_bit] =
            values;
        Self { input_state, raw_move, analog, sneak_mask, jump_mask, sprint_bit, up_bit, down_bit, left_bit, right_bit }
            .checked()
    }

    /// The sneak test holds SneakDown, and the jump mask and the five bit numbers are the expected
    /// ones. The jump and bit entries have one recipe each, so another value is not trusted: the
    /// flags are then neither read nor written, and only the vectors carry the move.
    pub fn flags_as_expected(&self) -> bool {
        self.sneak_mask & Self::SNEAK_DOWN != 0
            && self.jump_mask == Self::EXPECTED_JUMP_MASK
            && self.bits() == Self::EXPECTED_BITS
    }

    /// The move in `raw`: the analog vector `[left, forward]` when it is not zero, else the direction
    /// bits (1.0 or -1.0 per axis); the held actions from the flag word.
    pub fn read(&self, raw: &[u8; Self::COMPONENT_SIZE]) -> AmmlMoveIntent {
        let [mut left, mut forward] = self.vector_at(raw, self.analog_offset());
        let flags = self.flags(raw);
        let expected = self.flags_as_expected();
        let set = |bits: u32| expected && flags & bits != 0;
        if left == 0.0 && forward == 0.0 && expected {
            let axis = |plus: u32, minus: u32| f32::from(i8::from(set(plus)) - i8::from(set(minus)));
            forward = axis(Self::bit(self.up_bit), Self::bit(self.down_bit));
            left = axis(Self::bit(self.left_bit), Self::bit(self.right_bit));
        }
        AmmlMoveIntent {
            forward,
            left,
            jump: u8::from(set(self.jump_mask)),
            sneak: u8::from(set(Self::SNEAK_DOWN)),
            sprint: u8::from(set(Self::bit(self.sprint_bit))),
            _reserved: 0,
        }
    }

    /// Clears the move of `raw`: the owned held bits (when the flags are as expected), the analog
    /// vector and the raw move. Edge bits are not touched.
    pub fn zero(&self, raw: &mut [u8; Self::COMPONENT_SIZE]) {
        self.write(raw, AmmlMoveIntent { forward: 0.0, left: 0.0, jump: 0, sneak: 0, sprint: 0, _reserved: 0 });
    }

    /// Writes `intent` into `raw`: the owned held bits are cleared, then set from the intent (only
    /// when the flags are as expected); the analog vector and the raw move are `[left, forward]`,
    /// held to length 1. Edge bits ("was pressed") are never touched.
    pub fn write(&self, raw: &mut [u8; Self::COMPONENT_SIZE], intent: AmmlMoveIntent) {
        let mut vector = [intent.left, intent.forward];
        let length = (vector[0] * vector[0] + vector[1] * vector[1]).sqrt();
        if length > 1.0 {
            vector = [vector[0] / length, vector[1] / length];
        }
        if self.flags_as_expected() {
            let zone = Self::DIRECTION_DEAD_ZONE;
            let set = Self::when(intent.sneak != 0, Self::SNEAK_DOWN)
                | Self::when(intent.jump != 0, self.jump_mask)
                | Self::when(intent.sprint != 0, Self::bit(self.sprint_bit))
                | Self::when(intent.forward > zone, Self::bit(self.up_bit))
                | Self::when(intent.forward < -zone, Self::bit(self.down_bit))
                | Self::when(intent.left > zone, Self::bit(self.left_bit))
                | Self::when(intent.left < -zone, Self::bit(self.right_bit));
            let flags = (self.flags(raw) & !self.owned()) | set;
            Self::put(raw, self.input_state as usize, &flags.to_le_bytes());
        }
        let bytes = Self::vector_bytes(vector);
        Self::put(raw, self.analog_offset(), &bytes);
        Self::put(raw, self.raw_move as usize, &bytes);
    }

    fn checked(self) -> Option<Self> {
        let flags = (self.input_state, self.input_state.checked_add(Self::FLAGS_SIZE)?);
        let analog_start = self.input_state.checked_add(self.analog)?;
        let analog = (analog_start, analog_start.checked_add(Self::VEC2_SIZE)?);
        let raw_move = (self.raw_move, self.raw_move.checked_add(Self::VEC2_SIZE)?);
        let ranges = [flags, analog, raw_move];
        if ranges.iter().any(|(_, end)| *end as usize > Self::COMPONENT_SIZE) {
            return None;
        }
        let overlaps = |a: (u32, u32), b: (u32, u32)| a.0 < b.1 && b.0 < a.1;
        if overlaps(flags, analog) || overlaps(flags, raw_move) || overlaps(analog, raw_move) {
            return None;
        }
        if self.bits().iter().any(|bit| *bit >= u32::BITS) || self.sneak_mask == 0 || !self.jump_mask.is_power_of_two()
        {
            return None;
        }
        (self.owned().count_ones() == Self::OWNED_BITS).then_some(self)
    }

    fn bits(&self) -> [u32; 5] {
        [self.sprint_bit, self.up_bit, self.down_bit, self.left_bit, self.right_bit]
    }

    fn owned(&self) -> u32 {
        self.bits().iter().fold(Self::SNEAK_DOWN | self.jump_mask, |owned, bit| owned | Self::bit(*bit))
    }

    fn analog_offset(&self) -> usize {
        (self.input_state + self.analog) as usize
    }

    fn flags(&self, raw: &[u8; Self::COMPONENT_SIZE]) -> u32 {
        let start = self.input_state as usize;
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&raw[start..start + 4]);
        u32::from_le_bytes(bytes)
    }

    fn vector_at(&self, raw: &[u8; Self::COMPONENT_SIZE], start: usize) -> [f32; 2] {
        let float = |at: usize| {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&raw[at..at + 4]);
            f32::from_le_bytes(bytes)
        };
        [float(start), float(start + 4)]
    }

    fn vector_bytes(vector: [f32; 2]) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[..4].copy_from_slice(&vector[0].to_le_bytes());
        bytes[4..].copy_from_slice(&vector[1].to_le_bytes());
        bytes
    }

    // `checked` keeps every range inside the component.
    fn put(raw: &mut [u8; Self::COMPONENT_SIZE], start: usize, bytes: &[u8]) {
        raw[start..start + bytes.len()].copy_from_slice(bytes);
    }

    fn bit(number: u32) -> u32 {
        1u32.checked_shl(number).unwrap_or(0)
    }

    fn when(on: bool, bits: u32) -> u32 {
        if on { bits } else { 0 }
    }
}
