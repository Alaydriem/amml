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
