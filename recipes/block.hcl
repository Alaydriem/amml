entry "Block.canProvideFullSupport" {
  kind     = "function"
  notes    = "bool Block::canProvideFullSupport(uchar face) const, a non-virtual member: rcx = Block this, dl = the face (1 is up), the bool is returned in al. The body fetches a support component of the block (`mov rsi,rcx; call getComponent; test rax,rax; je`), returns false when the component value is 0, calls the component support function with the face when the value is 1, and otherwise forwards to BlockType::canProvideFullSupport(Block const&, uchar) with `mov rcx,[rsi+68h]; mov rdx,rsi; mov r8d,ebx` (the block type, the block and the face) as a tail jump. Block::canProvideSupport(uchar, BlockSupportType), the function before it in the image, has the same shape but a type argument and calls the BlockType virtual at slot 18 instead. Recipe 0 is the head of the body: the face saved in ebx, the component call, the null test and the compare of the value against 1, then function_start_of. Recipe 1 is one of the two call sites, in a function that tests a flags word and then calls with `mov rcx,rsi; mov dl,1` (the face up): the target of the call is the function. They are in two functions. Every register, displacement, immediate and branch byte is a wildcard. The check is function_start, which Recipe 1 can fail. In 6ad79ee4-17d51891a1119914 (26.60) the function is at 0x31533B0 with the same body (the component fetch, the 0/1 test and the tail jump to the BlockType function); both recipes derived on the cold run, with no change. Class: survived."

  recipe {
    step "pattern" {
      bytes = "89 D3 48 89 CE E8 ?? ?? ?? ?? 48 85 C0 74 ?? 8B 00 85 C0 74 ?? 83 F8 ?? 75 ?? 48 89 F1 89 DA"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "F7 81 ?? ?? ?? ?? ?? ?? ?? ?? 74 ?? 31 C0 EB ?? B0 ?? EB ?? 48 89 F1 B2 ?? E8 ?? ?? ?? ?? 48 8B 4C 24"
      offset = 25
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Block.tryGetComponent.BlockMapColorComponent" {
  kind     = "function"
  notes    = "BlockMapColorComponent const* Block::tryGetComponent<BlockMapColorComponent>() const, the template instance. Signature: rcx = block, the result in rax (null when the block has no map colour component). On 26.51 the body searches the component index vectors inline; on 26.60 it calls a lookup helper twice (block, then block type) and returns the pointer + 8. Its callers are the map-colour path: call this function, test the result, read `byte ptr [result+10h]` (mTintMethod) into ecx, call BiomeColorSampling.getMapPolicy, call MapPolicy.get and multiply the colour at result+0 by the tint; without the component they call BlockType.getMapColor. Function A is a map-colour function found by its own anchor, `lea r14,[rbx+..]; mov rcx,[rbx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,r14; call [guard]` (unique on 26.51 and 26.60; a byte-identical twin of A on 26.60 has the first two instructions swapped). Function B is a second map-colour function found by `mov [rbp+..],eax; mov rdi,[rdi+..]; mov rcx,rbx; call`: a pattern_any with one alternative per shape, because the store is disp32 on 26.51 and disp8 on 26.60 (the frame changed). Both anchors end at a call. The steps after the anchor scan only the function that holds it. Every displacement, immediate and branch byte is a wildcard. Recipe 0: function A, then `mov rbx,[rbx+..]; mov rcx,rax; call`, read as a branch target. Recipe 1: function B, then `mov rdi,[rdi+..]; mov rcx,rbx; call` (the end of the anchor), read as a branch target. They are in two functions. Check function_start. Evidence on 26.60: (1) the anchor of function A, drafted from the old function A, matches once on 26.60, and the call at the same place (before the getMapPolicy call) goes to the candidate; (2) the old function had 8 call sites in 6 functions, each followed by the getMapPolicy call; the candidate has 8 call sites in 7 functions, and every getMapPolicy call site on 26.60 follows a call to it, also in function B. What changed in 6ad79ee4-17d51891a1119914 (26.60): every call is followed by a nop, so each old pattern that ran past a call stopped matching; the old scripting map-colour function became a byte-identical twin of function A (same registers), so a pattern in it is no longer unique; the colour is now read with one movups/mulps instead of four movss. Class: real code change (call-site nops, register allocation, vectorised colour); patterns that end at a call would have survived the nops. Grade: two recipes in two functions plus function_start."

  recipe {
    step "pattern" { bytes = "4C 8D 73 ?? 48 8B 4B ?? 48 8B 01 48 8B 40 ?? 4C 89 F2 FF 15" }
    step "pattern" {
      bytes  = "48 8B 5B ?? 48 89 C1 E8"
      offset = 7
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = ["89 85 ?? ?? ?? ?? 48 8B 7F ?? 48 89 D9 E8", "89 45 ?? 48 8B 7F ?? 48 89 D9 E8"]
    }
    step "pattern" {
      bytes  = "48 8B 7F ?? 48 89 D9 E8"
      offset = 7
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Block.mDirectData.mLightEmission" {
  kind  = "field"
  notes = "Offset, from the start of the Block, of BlockComponentDirectData::mLightEmission (a 1-byte Brightness, 0..15, the light level the block emits), inside Block::mDirectData. LeviLamina headers world/level/block/Block.h and components/BlockComponentDirectData.h: the vtable (8), mComponents (60h), mBlockType at 68h, mCachedComponentData (8) at 70h, mDirectData at 78h; in mDirectData five component pointers (28h), four bools, then mLightEmission at +2Ch and mLight (the light the block absorbs) at +2Dh, so A4h and A5h from the Block. The game reads it from the Block, not through BlockType. Recipe 0 is the light spread core (the function that LightPropagationCoordinator.propagateForMesh calls first): for every block of a subchunk that is not air it loads the emission and the absorption, writes the absorption into a 4-bit-per-block array and, when the emission is not 0, pushes the block, its position and the emission byte (+0Bh of a 10h record) into the emitter list. It is a pattern_any with one alternative per shape, both starting at the emission load at offset 0, read as the displacement of operand 1. Alternative 0 (26.60, 26.62, 26.63): `movzx r9d,byte ptr [r14+..]; movzx eax,byte ptr [r14+..]; mov ecx,eax; or cl,r9b; je`; the fixed `mov ecx,eax; or cl,r9b` bytes tie the first load (r9b, the emitter byte) and the second (al, the absorption written to the array). Alternative 1 (Preview): the absorption, a new byte at +71h and the emission are loaded in that order, and the pattern starts at the third load: `movzx edx,byte ptr [rdi+..]; mov eax,ecx; mov [rbp+..],dl; or al,dl; setne al; or al,r9b; cmp al,..; jne`; the emission is kept in a frame slot and pushed into the emitter list from there. Recipe 1 is the point light function of Vibrant Visuals (26.62 0x790b9d0, the function just before PointLightCoordinator.getAnalyticalLights in the image): it loads the emission of the block before it asks the block for a component (`movzx r14d,byte ptr [rdi+..]; mov rcx,rdi; call`) and keeps it for the light it adds. Step 0 is a pattern_any that finds that function, one alternative per shape: `movaps xmm7,xmm8; movzx; mov; call` (26.60..26.63) and `xor r14d,r14d; movzx; mov; call` (Preview); step 1 scans only that function for `movzx r..,byte ptr [r..+disp32]; mov rcx,r..; call` (once in it on every build), offset 0, read as the displacement of operand 1. The two recipes are in two functions and share no pattern bytes; each pattern ends at a call or before one. Every displacement, immediate and branch byte is a wildcard. Range window 80h..100h. Values: A4h on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63); A5h on 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23), where the members after it moved one byte (the absorption is at A6h) and the code also reads a new byte at +71h. On 6aa482fd-2658118eba0dbac5 (26.51) neither recipe matches and the entry refuses; 26.51 is out of scope. Evidence: (1) on 26.63 the emitter list of the spread core is filled from +A4h and the absorption array from +A5h; on the Preview the same loop (the first direct call of propagateForMesh, the same per-block virtual getBlock and air test) fills the emitter list from +A5h and the absorption array from +A6h; (2) the point light function, found on the Preview by the same body after the load (`call; nop; movaps xmm0,[rip+..]; movss xmm8,[rip+..]; test rax,rax; je; mov eax,[rax+10h]; cmp eax,2; je; cmp eax,71h; jne`), reads +A4h on 26.62 and +A5h on the Preview. Also on 26.60..26.63: LevelChunk::_lightingCallbacks (26.63 0x384e460) compares the old and the new block at +A4h and +A5h before it calls the lighting update, and Item::getLightEmission(int) (Item vtable slot 121 on 26.63, 0x3f976b0) returns the byte at +A4h of the item's block state. What changed in 6ae4eeee-4dd4fa7acacb4c41: the member moved one byte and the spread loop has a new shape; the meaning is the same."

  recipe {
    step "pattern_any" {
      patterns = [
        "45 0F B6 ?? ?? ?? ?? ?? 41 0F B6 ?? ?? ?? ?? ?? 89 C1 44 08 C9 74 ??",
        "0F B6 ?? ?? ?? ?? ?? 89 C8 88 95 ?? ?? ?? ?? 08 D0 0F 95 C0 44 08 C8 3C ?? 75 ??",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "41 0F 28 F8 44 0F B6 ?? ?? ?? ?? ?? 48 89 ?? E8",
        "45 31 F6 44 0F B6 ?? ?? ?? ?? ?? 48 89 ?? E8",
      ]
    }
    step "pattern" { bytes = "44 0F B6 ?? ?? ?? ?? ?? 48 89 ?? E8" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 256
  }
}
